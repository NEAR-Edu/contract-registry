use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use warp::{reject::Reject, Rejection};

use super::client::request_job;

#[derive(Serialize, Deserialize)]
pub struct WebhookPayloadJob {
    pub name: String,
    pub status: String,
    pub number: u64,
}

#[derive(Serialize, Deserialize)]
pub struct JobCompletedWebhookPayload {
    pub job: WebhookPayloadJob,
}

#[derive(Debug, Error)]
pub enum WebhookError {
    #[error("Error parsing JSON body: {0}")]
    PayloadParseError(#[from] serde_json::Error),
}

impl Reject for WebhookError {}

pub async fn handler(
    client: Client,
    project_slug: String,
    body: warp::hyper::body::Bytes,
) -> Result<String, Rejection> {
    let payload = serde_json::from_slice::<JobCompletedWebhookPayload>(&body)
        .map_err(|e| WebhookError::from(e))?;
    let job_number = payload.job.number.to_string();
    let meta = request_job(&client, project_slug, job_number).await?;
    Ok(format!("{}", meta.code_hash))
}
