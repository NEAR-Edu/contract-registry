use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::types::{AccountId, BlockReference, Finality, FunctionArgs};
use near_primitives::views::QueryRequest;

use serde_json::from_slice;
use thiserror::Error;
use tokio::time;

use crate::network_config::NetworkConfig;

#[derive(Debug, Error)]
pub enum ContractInteractionError {
    #[error("Incompatible response type")]
    IncompatibleResponseType(QueryResponseKind),
}

pub async fn poll(
    network_config: &NetworkConfig,
    contract_id: AccountId,
    method_name: String,
    args: &serde_json::Value,
) {
    let mut interval = time::interval(time::Duration::from_secs(5));
    loop {
        interval.tick().await;
        println!("Viewing...");
        let value = view(
            network_config,
            contract_id.clone(),
            method_name.clone(),
            args,
        )
        .await
        .unwrap();
        println!("Got: {}", value.as_str().unwrap());
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
        // println!("{:#?}", String::from_utf8(result.result.clone()));
        from_slice(&result.result[..]).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    } else {
        Err(Box::new(
            ContractInteractionError::IncompatibleResponseType(response.kind),
        ))
    }
}
