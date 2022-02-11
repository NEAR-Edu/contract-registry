use dotenv;
use std::env::var;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::Filter;

mod circleci_artifacts;
mod circleci_verify;
mod env;
use crate::{env::CIRCLECI_WEBHOOK_SECRET, circleci_artifacts::VerifierClient};

#[tokio::main]
async fn main() {
  if let Err(_) = dotenv::dotenv() {
    println!("No .env file found.");
  }

  let project_slug = std::env::var(env::CIRCLECI_PROJECT_SLUG).unwrap();
  let api_key = std::env::var(env::CIRCLECI_API_KEY).unwrap();

  let vclient = VerifierClient::new(project_slug, &api_key);
  let artifacts = vclient.get_job_artifacts("24").await.unwrap();
  println!("{:?}", &artifacts);
  let metadata = vclient.assemble(artifacts).await.unwrap();
  println!("{:?}", &metadata);

  // vclient.

  // let artifacts = circleci_artifacts::get_job_artifacts(
  //   &project_slug,
  //   "24",
  //   &api_key,
  // )
  // .await;
  // println!("{:?}", artifacts);

  return;

  let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "tracing=info,warp=debug".to_owned());

  tracing_subscriber::fmt()
    .with_env_filter(filter)
    .with_span_events(FmtSpan::CLOSE)
    .init();

  let circleci_webhook_secret = var(CIRCLECI_WEBHOOK_SECRET).unwrap();

  let guarded = warp::path!("circle")
    .and(circleci_verify::verify_filter(&circleci_webhook_secret))
    .map(|| "ok without pass!");

  let routes = guarded.with(warp::trace::request());

  warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}
