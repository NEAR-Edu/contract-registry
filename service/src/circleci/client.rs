use futures::{future, Future};
use model::code_hash::CodeHash;
use std::collections::HashMap;
use thiserror::Error;
use tokio::task::JoinError;
use warp::reject::Reject;

use reqwest::Client;

use super::error::CircleCiError;

pub async fn request_job(
    client: &Client,
    project_slug: String,
    job_number: String,
) -> Result<VerificationMetadata, ParallelError<CircleCiError>> {
    let artifacts = get_job_artifacts(client, &project_slug, &job_number).await?;
    let metadata = assemble(client, artifacts).await.map_err(|e| match e {
        ParallelError::TaskError(f) => ParallelError::TaskError(CircleCiError::ReqwestError(f)),
        ParallelError::JoinError(f) => ParallelError::JoinError(f),
    })?;
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

#[derive(Error, Debug)]
pub enum ParallelError<E: std::error::Error> {
    #[error("Join error: {0}")]
    JoinError(JoinError),
    #[error("Task error: {0}")]
    TaskError(#[from] E),
}

impl<E: std::error::Error + Send + Sync + 'static> Reject for ParallelError<E> {}

pub async fn parallel_map<T, I, F, O, V, E>(items: I, f: F) -> Result<Vec<V>, ParallelError<E>>
where
    T: Send + Sync + 'static,
    I: IntoIterator<Item = T>,
    F: Fn(T) -> O,
    O: Future<Output = Result<V, E>> + Send + Sync + 'static,
    V: Send + Sync + 'static,
    E: std::error::Error + Send + Sync + 'static,
{
    Ok(future::join_all(items.into_iter().map(|item| {
        let fut = f(item);
        tokio::spawn(async move { fut.await })
    }))
    .await
    .into_iter()
    .collect::<Result<Vec<_>, JoinError>>()
    .map_err(|e| ParallelError::JoinError(e))?
    .into_iter()
    .collect::<Result<Vec<_>, E>>()?)
}

pub async fn assemble(
    client: &Client,
    artifacts: HashMap<String, String>,
) -> Result<VerificationMetadata, ParallelError<reqwest::Error>> {
    let responses = parallel_map(
        vec![
            "git/repository.txt",
            "git/remote.txt",
            "git/branch.txt",
            "git/commit.txt",
        ],
        |path| {
            let url = &artifacts[path];
            client.get(url).send()
        },
    )
    .await?;
    let to_text = parallel_map(responses, |r| r.text()).await?;

    let (repository, remote, branch, commit) = match &to_text[..] {
        [a, b, c, d] => (a, b, c, d),
        _ => unreachable!(),
    };

    let code_url = (&artifacts["out/out.wasm"]).to_string();
    let wasm = client.get(&code_url).send().await?.bytes().await?;

    let code = wasm.as_ref().to_vec();
    let code_hash = CodeHash::hash_bytes(&code);
    Ok(VerificationMetadata {
        repo: repository.trim().to_string(),
        remote: remote.trim().to_string(),
        branch: branch.trim().to_string(),
        commit: commit.trim().to_string(),
        code_url,
        code_hash,
    })
}

#[derive(Debug)]
pub struct VerificationMetadata {
    pub repo: String,
    pub remote: String,
    pub branch: String,
    pub commit: String,
    pub code_url: String,
    pub code_hash: CodeHash,
}
