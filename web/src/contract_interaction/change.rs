use near_crypto::{InMemorySigner, Signer};
use near_jsonrpc_client::{
    methods,
    JsonRpcClient,
};

use near_primitives::{
    transaction::{Action, FunctionCallAction, Transaction},
    types::AccountId,
};

use super::{valid_for, wait_for_status};

pub async fn change(
    client: &JsonRpcClient,
    signer: &InMemorySigner,
    contract_id: &AccountId,
    method: &str,
    args: serde_json::Value,
    deposit: u128,
) -> Result<String, Box<dyn std::error::Error>> {
    let (nonce, block_hash) = valid_for(client, signer).await?;

    let tx = Transaction {
        signer_id: signer.account_id.clone(),
        public_key: signer.public_key.clone(),
        nonce: nonce + 1,
        receiver_id: contract_id.clone(),
        block_hash,
        actions: vec![Action::FunctionCall(FunctionCallAction {
            method_name: method.to_string(),
            args: args.to_string().into_bytes(),
            gas: 100_000_000_000_000, // 100 TGas
            deposit,
        })],
    };

    println!("Signing with {}", signer.public_key());
    let request = methods::broadcast_tx_async::RpcBroadcastTxAsyncRequest {
        signed_transaction: tx.sign(signer),
    };

    let tx_hash = client.call(request).await?;

    println!("Sent transaction {}", tx_hash);

    wait_for_status(client, &signer.account_id, tx_hash).await
}
