use dotenv;
use near_crypto::{InMemorySigner, SecretKey};
use near_jsonrpc_client::JsonRpcClient;
use near_primitives::types::AccountId;
use serde_json::json;
use std::{env::var, str::FromStr};
use tracing_subscriber::fmt::format::FmtSpan;
use warp::Filter;

use crate::{
    circleci::verify::verify_filter, contract_interaction::change::change,
    env::CIRCLECI_WEBHOOK_SECRET,
};

mod circleci;
mod contract_interaction;
mod env;
mod network_config;

#[tokio::main]
async fn main() {
    if let Err(_) = dotenv::dotenv() {
        println!("No .env file found.");
    }

    let network_config_path = std::env::var(env::NETWORK_CONFIG).unwrap();

    let network_config = network_config::load(&network_config_path);

    println!(
        "Connecting to {} at {}...",
        network_config.network_id, network_config.node_url
    );

    let client = JsonRpcClient::connect(network_config.node_url);
    let account_id: AccountId = std::env::var(env::ACCOUNT_ID).unwrap().parse().unwrap();
    let contract_id: AccountId = std::env::var(env::CONTRACT_ID).unwrap().parse().unwrap();
    let secret_key = SecretKey::from_str(&std::env::var(env::SECRET_KEY).unwrap()).unwrap();
    let signer = InMemorySigner::from_secret_key(account_id, secret_key);

    let value = change(
        &client,
        &signer,
        &contract_id,
        "verification_failure",
        json!({"id":"0"}),
        1,
    )
    .await;

    println!("Value: {:?}", value);

    // let mut rx: Receiver<VerificationRequest> = contract_interaction::watch::list(
    //     network_config,
    //     std::env::var(env::CONTRACT_ID).unwrap().parse().unwrap(),
    //     "get_pending_requests".to_string(),
    //     json!({}),
    //     time::Duration::from_secs(5),
    // );

    // println!("Before loop");
    // while let Some(v) = rx.recv().await {
    //     println!("Received: {:?}", v);
    // }
    // println!("After loop");

    // let x = contract_interaction::call(&network_config).await;

    // let project_slug = std::env::var(env::CIRCLECI_PROJECT_SLUG).unwrap();
    // let api_key = std::env::var(env::CIRCLECI_API_KEY).unwrap();

    // let vclient = VerifierClient::new(&project_slug, &api_key);
    // let artifacts = vclient.get_job_artifacts("24").await.unwrap();
    // println!("{:?}", &artifacts);
    // let metadata = vclient.assemble(artifacts).await.unwrap();
    // println!("{}", &metadata.code_hash);

    return;

    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "tracing=info,warp=debug".to_owned());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let circleci_webhook_secret = var(CIRCLECI_WEBHOOK_SECRET).unwrap();

    let guarded = warp::path!("circle")
        .and(verify_filter(&circleci_webhook_secret))
        .map(|| "ok without pass!");

    let routes = guarded.with(warp::trace::request());

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}
