use dotenv;
use near_jsonrpc_client::header::HeaderValue;
use reqwest::{header::HeaderMap, Client};
use std::{convert::Infallible, env::var, str::FromStr};
use tracing_subscriber::fmt::format::FmtSpan;
use warp::Filter;

use crate::{
    circleci::{
        signature::verify_filter,
        webhook::{self, JobCompletedWebhookPayload},
    },
    env::CIRCLECI_WEBHOOK_SECRET,
};

mod circleci;
mod contract_interaction;
mod env;
mod network_config;
mod repository;

const TOKEN_HEADER: &'static str = "Circle-Token";

fn with<T: Clone + Send>(w: T) -> impl Filter<Extract = (T,), Error = Infallible> + Clone {
    warp::any().map(move || w.clone())
}

fn create_circleci_reqwest_client() -> Client {
    let api_key = std::env::var(env::CIRCLECI_API_KEY).unwrap();

    let mut headers = HeaderMap::new();
    let mut api_key_header_value = HeaderValue::from_str(&api_key).unwrap();
    api_key_header_value.set_sensitive(true);
    headers.insert(TOKEN_HEADER, api_key_header_value);
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    client
}

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

    // let rpc_client = JsonRpcClient::connect(network_config.node_url);
    // let account_id: AccountId = std::env::var(env::ACCOUNT_ID).unwrap().parse().unwrap();
    // let contract_id: AccountId = std::env::var(env::CONTRACT_ID).unwrap().parse().unwrap();
    // let secret_key = SecretKey::from_str(&std::env::var(env::SECRET_KEY).unwrap()).unwrap();
    // let signer = InMemorySigner::from_secret_key(account_id, secret_key);

    // let value = change(
    //     &rpc_client,
    //     &signer,
    //     &contract_id,
    //     "verification_failure",
    //     json!({"id":"0"}),
    //     1,
    // )
    // .await;

    // println!("Value: {:?}", value);

    let project_slug = std::env::var(env::CIRCLECI_PROJECT_SLUG).unwrap();

    let circleci_reqwest_client = create_circleci_reqwest_client();

    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "tracing=info,warp=debug".to_owned());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let circleci_webhook_secret = var(CIRCLECI_WEBHOOK_SECRET).unwrap();

    let guarded = warp::path!("webhook")
        .and(warp::body::content_length_limit(1024 * 32 /* 32kb */))
        .and(with(circleci_reqwest_client))
        .and(with(project_slug))
        .and(verify_filter(circleci_webhook_secret))
        .and_then(webhook::handler);

    let routes = guarded.with(warp::trace::request());

    warp::serve(routes)
        .run((
            [127, 0, 0, 1],
            std::env::var(env::PORT)
                .ok()
                .and_then(|s| u16::from_str(&s).ok())
                .unwrap_or(8000),
        ))
        .await;
}
