use near_jsonrpc_client::{self, methods};
use near_primitives::{types::{BlockReference, Finality, FunctionArgs}, views::QueryRequest};
use serde_json::json;

pub fn r() {
    let request = methods::query::RpcQueryRequest {
        block_reference: BlockReference::Finality(Finality::Final),
        request: QueryRequest::CallFunction {
            account_id: "nosedive.testnet".parse().unwrap(),
            method_name: "status".to_string(),
            args: FunctionArgs::from(
                json!({
                    "account_id": "miraclx.testnet",
                })
                .to_string()
                .into_bytes(),
            ),
        },
    };
}
