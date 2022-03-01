use core::time;
use dotenv;
use model::verification::VerificationRequest;
use serde_json::json;
use std::env::var;
use tokio::sync::mpsc::{self, Receiver};
use tracing_subscriber::fmt::format::FmtSpan;
use warp::Filter;

use crate::{
    circleci::{artifacts::VerifierClient, verify::verify_filter},
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

    // let (tx, mut rx) = mpsc::channel::<VerificationRequest>(16);

    // tokio::spawn(async move {
    let mut rx: Receiver<VerificationRequest> = contract_interaction::watch::list(
        network_config,
        std::env::var(env::CONTRACT_ID).unwrap().parse().unwrap(),
        "get_pending_requests".to_string(),
        json!({}),
        time::Duration::from_secs(5),
    );
    // });

    println!("Before loop");
    while let Some(v) = rx.recv().await {
        println!("Received: {:?}", v);
    }
    println!("After loop");

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
