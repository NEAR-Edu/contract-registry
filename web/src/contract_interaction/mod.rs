use near_jsonrpc_primitives::types::query::QueryResponseKind;

use thiserror::Error;

pub mod watch;

#[derive(Debug, Error)]
pub enum ContractInteractionError {
    #[error("Incompatible response type from RPC {0:?}")]
    IncompatibleRpcResponseType(QueryResponseKind),
}
