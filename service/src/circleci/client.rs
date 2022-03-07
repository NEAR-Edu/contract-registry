use model::code_hash::CodeHash;
use std::collections::HashMap;

use reqwest::Client;

use super::error::CircleCiError;

pub async fn request_job(
    client: &Client,
    project_slug: String,
    job_number: String,
) -> Result<VerificationMetadata, CircleCiError> {
    let artifacts = get_job_artifacts(client, &project_slug, &job_number).await?;
    let metadata = assemble(client, artifacts).await?;
    Ok(metadata)
}

pub async fn get_job_artifacts(
    client: &Client,
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

pub async fn assemble(
    client: &Client,
    artifacts: HashMap<String, String>,
) -> Result<VerificationMetadata, reqwest::Error> {
    let repo = client
        .get(&artifacts["git/repo.txt"])
        .send()
        .await?
        .text()
        .await?;
    let remote = client
        .get(&artifacts["git/remote.txt"])
        .send()
        .await?
        .text()
        .await?;
    let branch = client
        .get(&artifacts["git/branch.txt"])
        .send()
        .await?
        .text()
        .await?;
    let commit = client
        .get(&artifacts["git/commit.txt"])
        .send()
        .await?
        .text()
        .await?;
    let wasm = client
        .get(&artifacts["out/out.wasm"])
        .send()
        .await?
        .bytes()
        .await?;

    let code = wasm.as_ref().to_vec();
    let code_hash = CodeHash::hash_bytes(&code);
    Ok(VerificationMetadata {
        repo: repo.trim().to_string(),
        remote: remote.trim().to_string(),
        branch: branch.trim().to_string(),
        commit: commit.trim().to_string(),
        code,
        code_hash,
    })
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
