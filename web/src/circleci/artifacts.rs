use model::code_hash::CodeHash;
use std::collections::HashMap;
use thiserror::Error;

use futures::{stream, StreamExt};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};

pub struct VerifierClient<'a> {
    project_slug: &'a str,
    client: Client,
}

impl<'a> VerifierClient<'a> {
    pub fn new(project_slug: &'a str, api_key: &str) -> Self {
        let mut headers = HeaderMap::new();
        let mut api_key_header_value = HeaderValue::from_str(api_key).unwrap();
        api_key_header_value.set_sensitive(true);
        headers.insert("Circle-Token", api_key_header_value);
        let client = Client::builder().default_headers(headers).build().unwrap();

        Self {
            client,
            project_slug,
        }
    }

    pub async fn get_job_artifacts(
        &self,
        job_number: &str,
    ) -> Result<HashMap<String, String>, String> {
        let json = self
            .client
            .get(format!(
                "https://circleci.com/api/v2/project/{}/{}/artifacts",
                &self.project_slug, job_number
            ))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<serde_json::Value>()
            .await
            .map_err(|e| e.to_string())?;

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
            .ok_or("Failed to parse JSON".to_string())
    }

    pub async fn assemble(
        &self,
        artifacts: HashMap<String, String>,
    ) -> Result<VerificationMetadata, reqwest::Error> {
        let requests = stream::iter(vec![
            "git/repo.txt",
            "git/remote.txt",
            "git/branch.txt",
            "git/commit.txt",
        ])
        .map(|p| {
            let client = &self.client;
            let url = &artifacts[p];
            async move { client.get(url).send().await?.text().await }
        })
        .buffer_unordered(2)
        .collect::<Vec<Result<String, reqwest::Error>>>()
        .await
        .into_iter()
        .collect::<Result<Vec<String>, reqwest::Error>>()?;

        let code_download = self
            .client
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
}

#[derive(Error, Debug)]
pub enum AssembleError {
    #[error("Could not download metadata: {0:?}")]
    MetadataDownload(#[from] reqwest::Error),
}

#[derive(Debug)]
pub struct VerificationMetadata {
    pub repo: String,
    pub remote: String,
    pub branch: String,
    pub commit: String,
    pub code: Vec<u8>,
    pub code_hash: CodeHash,
}
