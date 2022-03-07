use std::io::Repeat;

use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use warp::{Rejection, Reply};

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

pub async fn handler(
    client: Client,
    project_slug: String,
    payload: JobCompletedWebhookPayload,
) -> Result<String, Rejection> {
    let job_number = payload.job.number.to_string();
    let meta = request_job(&client, project_slug, job_number).await?;
    Ok(format!("{}", meta.code_hash))
}
