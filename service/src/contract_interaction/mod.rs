use std::time;

use near_crypto::{InMemorySigner, Signer};
use near_jsonrpc_client::{
    methods::{
        self, broadcast_tx_commit::RpcTransactionError, query::RpcQueryRequest, tx::TransactionInfo,
    },
    JsonRpcClient,
};
use near_jsonrpc_primitives::types::query::QueryResponseKind;

use near_primitives::{
    hash::CryptoHash,
    types::{AccountId, BlockReference},
    views::{FinalExecutionOutcomeView, FinalExecutionStatus, QueryRequest},
};
use thiserror::Error;

pub mod change;
pub mod view;
pub mod watch;

#[derive(Error, Debug)]
pub enum TransactionStatusError {
    #[error("JSON RPC error: {0}")]
    JsonRpcError(#[from] near_jsonrpc_client::errors::JsonRpcError<RpcTransactionError>),
    #[error("Execution error: {0}")]
    ExecutionError(near_primitives::errors::TxExecutionError),
}

pub async fn valid_for(
    client: &JsonRpcClient,
    signer: &InMemorySigner,
) -> Result<(u64, CryptoHash), Box<dyn std::error::Error>> {
    let res = client
        .call(RpcQueryRequest {
            block_reference: BlockReference::latest(),
            request: QueryRequest::ViewAccessKey {
                account_id: signer.account_id.clone(),
                public_key: signer.public_key(),
            },
        })
        .await?;

    match res.kind {
        QueryResponseKind::AccessKey(key) => Ok((key.nonce, res.block_hash)),
        _ => Err("Failed to parse response")?,
    }
}

pub async fn wait_for_status(
    client: &JsonRpcClient,
    account_id: &AccountId,
    hash: CryptoHash,
) -> Result<String, Box<dyn std::error::Error>> {
    loop {
        let response = client
            .call(methods::tx::RpcTransactionStatusRequest {
                transaction_info: TransactionInfo::TransactionId {
                    hash,
                    account_id: account_id.clone(),
                },
            })
            .await;

        match response {
            Err(err) => match err.handler_error()? {
                methods::tx::RpcTransactionError::UnknownTransaction { .. } => {
                    tokio::time::sleep(time::Duration::from_secs(2)).await;
                    continue;
                }
                err => Err(err)?,
            },
            Ok(FinalExecutionOutcomeView {
                status: FinalExecutionStatus::Failure(e),
                ..
            }) => {
                return Err(Box::new(TransactionStatusError::ExecutionError(e)));
            }
            Ok(FinalExecutionOutcomeView {
                status: FinalExecutionStatus::SuccessValue(s),
                ..
            }) => {
                return Ok(s);
            }
            _ => {
                // Transaction is currently executing
                tokio::time::sleep(time::Duration::from_secs(2)).await;
                continue;
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum ContractInteractionError {
    #[error("Incompatible response type from RPC {0:?}")]
    IncompatibleRpcResponseType(QueryResponseKind),
}

#[cfg(test)]
mod tests {
    use near_crypto::{KeyType, SecretKey};

    #[test]
    fn generate_ed25519_key() {
        let k = SecretKey::from_random(KeyType::ED25519);
        println!("Secret key: {}", k.to_string());
        println!("Verify key: {}", k.public_key().to_string());
    }
}
