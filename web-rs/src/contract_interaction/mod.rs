use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::types::{BlockReference, Finality, FunctionArgs};
use near_primitives::views::QueryRequest;

use serde::Deserialize;
use serde_json::{from_slice, json};

use crate::network_config::NetworkConfig;

pub async fn call(network_config: &NetworkConfig) -> Result<(), Box<dyn std::error::Error>> {
    let client = JsonRpcClient::connect(&network_config.node_url);

    let account_id = "miraclx.testnet";

    let request = methods::query::RpcQueryRequest {
        block_reference: BlockReference::Finality(Finality::Final),
        request: QueryRequest::CallFunction {
            account_id: "nosedive.testnet".parse()?,
            method_name: "status".to_string(),
            args: FunctionArgs::from(
                json!({
                    "account_id": account_id,
                })
                .to_string()
                .into_bytes(),
            ),
        },
    };


    let response = client.call(request).await?;

    if let QueryResponseKind::CallResult(result) = response.kind {
        println!("{:#?}", String::from_utf8(result.result));
    }

    Ok(())
}
