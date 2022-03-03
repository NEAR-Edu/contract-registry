use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::types::{AccountId, BlockReference, Finality, FunctionArgs};
use near_primitives::views::QueryRequest;

use serde_json::from_slice;

use crate::network_config::NetworkConfig;

use super::ContractInteractionError;

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
