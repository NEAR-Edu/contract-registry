use model::sequential_id::SequentialId;
use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::types::{AccountId, BlockReference, Finality, FunctionArgs};
use near_primitives::views::QueryRequest;

use serde::de::DeserializeOwned;
use serde_json::from_slice;
use thiserror::Error;
use tokio::sync::mpsc::Sender;
use tokio::time;

use crate::network_config::NetworkConfig;

#[derive(Debug, Error)]
pub enum ContractInteractionError {
    #[error("Incompatible response type from RPC {0:?}")]
    IncompatibleRpcResponseType(QueryResponseKind),
}

pub async fn poll_for_new<T, U>(
    network_config: &NetworkConfig,
    contract_id: AccountId,
    method_name: String,
    args: &serde_json::Value,
    tx: Sender<T>,
    duration: time::Duration,
) where
    T: SequentialId<U> + DeserializeOwned,
    U: Ord,
{
    let mut interval = time::interval(duration);
    // To ensure unique items are delivered from array, keep track of the
    // "largest" item delivered thus far.
    // Assumes that new items will be "larger" than old items.
    let mut largest_overall: Option<U> = None;
    loop {
        interval.tick().await;
        let mut largest_in_round: Option<U> = None;
        let items = view(
            network_config,
            contract_id.clone(),
            method_name.clone(),
            args,
        )
        .await
        .ok()
        .as_ref()
        .and_then(|view| view.as_array())
        .map(|arr| {
            arr.into_iter()
                .map(|item| serde_json::from_value::<T>(item.clone()))
                .filter_map(|item| match item {
                    Err(ref e) => {
                        // May be intentional (e.g. filter by parse-ability)
                        println!("Error parsing item: {}", e);
                        None
                    }
                    Ok(i) => Some(i),
                })
                .filter(|item| {
                    // Only take items that are "larger" than those we've seen already
                    largest_overall
                        .as_ref()
                        .map_or(true, |largest_overall| &item.seq_id() > largest_overall)
                })
                .collect::<Vec<T>>()
        });

        if let Some(items) = items {
            for item in items {
                // Update largest_in_round for every item
                if let Some(ref l) = largest_in_round {
                    if &item.seq_id() > l {
                        largest_in_round = Some(item.seq_id());
                    }
                } else {
                    largest_in_round = Some(item.seq_id());
                }

                match tx.send(item).await {
                    Ok(()) => {}
                    Err(e) => println!("Error sending across channel: {}", e),
                }
            }
        }

        // Only update largest_overall after processing each new item
        if largest_in_round.is_some() {
            largest_overall = largest_in_round;
        }
    }
}

pub async fn view(
    network_config: &NetworkConfig,
    contract_id: AccountId,
    method_name: String,
    args: &serde_json::Value,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = JsonRpcClient::connect(&network_config.node_url);

    let request = methods::query::RpcQueryRequest {
        block_reference: BlockReference::Finality(Finality::Final),
        request: QueryRequest::CallFunction {
            account_id: contract_id,
            method_name,
            args: FunctionArgs::from(args.to_string().into_bytes()),
        },
    };

    let response = client.call(request).await?;

    if let QueryResponseKind::CallResult(result) = response.kind {
        from_slice(&result.result[..]).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    } else {
        Err(Box::new(
            ContractInteractionError::IncompatibleRpcResponseType(response.kind),
        ))
    }
}
