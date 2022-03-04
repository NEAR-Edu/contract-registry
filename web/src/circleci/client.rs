use model::code_hash::CodeHash;
use std::collections::HashMap;
use thiserror::Error;
use warp::{Rejection, Reply};

use futures::{stream, StreamExt};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};

use super::{error::CircleCiError, webhook::JobCompletedWebhookPayload};

const TOKEN_HEADER: &'static str = "Circle-Token";

// #[derive(Clone)]
// pub struct VerifierClient {
//     project_slug: String,
//     client: Client,
// }

// impl VerifierClient {
//     pub fn new(project_slug: String, api_key: &str) -> Self {
//         let mut headers = HeaderMap::new();
//         let mut api_key_header_value = HeaderValue::from_str(api_key).unwrap();
//         api_key_header_value.set_sensitive(true);
//         headers.insert(TOKEN_HEADER, api_key_header_value);
//         let client = Client::builder().default_headers(headers).build().unwrap();

//         Self {
//             client,
//             project_slug,
//         }
//     }

pub async fn request_job(
    client: Client,
    project_slug: String,
    job_number: String,
) -> Result<VerificationMetadata, CircleCiError> {
    let artifacts = get_job_artifacts(client.clone(), &project_slug, &job_number).await?;
    let metadata = assemble(client, artifacts).await?;
    Ok(metadata)
}

async fn get_job_artifacts(
    client: Client,
    project_slug: &str,
    job_number: &str,
) -> Result<HashMap<String, String>, CircleCiError> {
    let json = client
        .get(format!(
            "https://circleci.com/api/v2/project/{}/{}/artifacts",
            project_slug, job_number
        ))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    json.as_object()
        .and_then(|json_obj| json_obj.get("items"))
        .and_then(|items_value| items_value.as_array())
        .map(|items_arr| {
            items_arr
                .iter()
                .flat_map(|item_value| {
                    item_value.as_object().and_then(|item_obj| {
                        let path = item_obj
                            .get("path")
                            .and_then(|path_value| path_value.as_str());
                        let url = item_obj.get("url").and_then(|url_value| url_value.as_str());
                        path.zip(url)
                            .map(|(path, url)| (path.to_string(), url.to_string()))
                    })
                })
                .collect::<HashMap<String, String>>()
        })
        .ok_or(CircleCiError::JsonSchemaMismatch)
}

async fn assemble(
    client: Client,
    artifacts: HashMap<String, String>,
) -> Result<VerificationMetadata, reqwest::Error> {
    let requests = stream::iter(vec![
        "git/repo.txt",
        "git/remote.txt",
        "git/branch.txt",
        "git/commit.txt",
    ])
    .map(|p| {
        let client = &client;
        let url = &artifacts[p];
        async move { client.get(url).send().await?.text().await }
    })
    .buffer_unordered(2)
    .collect::<Vec<Result<String, reqwest::Error>>>()
    .await
    .into_iter()
    .collect::<Result<Vec<String>, reqwest::Error>>()?;

    let code_download = client
        .get(&artifacts["out/out.wasm"])
        .send()
        .await?
        .bytes()
        .await?;

    if let [repo, remote, branch, commit] = &requests[..] {
        let code = code_download.as_ref().to_vec();
        let code_hash = CodeHash::hash_bytes(&code);
        Ok(VerificationMetadata {
            repo: repo.trim().to_string(),
            remote: remote.trim().to_string(),
            branch: branch.trim().to_string(),
            commit: commit.trim().to_string(),
            code,
            code_hash,
        })
    } else {
        // Should be able to deconstruct the same number of items as were requested
        unreachable!();
    }
}

//     pub async fn webhook_handler(
//         &self,
//         payload: JobCompletedWebhookPayload,
//     ) -> Result<impl Reply, Rejection> {
//         let job_number = payload.job.number.to_string();
//         let meta = self.request_job(&job_number).await?;
//         Ok(format!("{}", meta.code_hash))
//     }
// }

#[derive(Debug)]
pub struct VerificationMetadata {
    pub repo: String,
    pub remote: String,
    pub branch: String,
    pub commit: String,
    pub code: Vec<u8>,
    pub code_hash: CodeHash,
}
