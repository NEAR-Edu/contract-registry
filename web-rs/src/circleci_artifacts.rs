use std::collections::HashMap;

use futures::{
  stream::{self, BufferUnordered},
  StreamExt,
};
use reqwest::{
  header::{HeaderMap, HeaderValue},
  Client,
};

pub struct VerifierClient {
  project_slug: String,
  client: Client,
}

impl VerifierClient {
  pub fn new(project_slug: String, api_key: &str) -> Self {
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

    json
      .as_object()
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
              path
                .zip(url)
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
  ) -> Result<VerificationMetadata, Vec<reqwest::Error>> {
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
    .await;

    match requests[..] {
      [Ok(ref repo), Ok(ref remote), Ok(ref branch), Ok(ref commit)] => {
        Ok(VerificationMetadata {
          repo: repo.trim().to_string(),
          remote: remote.trim().to_string(),
          branch: branch.trim().to_string(),
          commit: commit.trim().to_string(),
        })
      }
      _ => Err(requests.into_iter().filter_map(|r| r.err()).collect()),
    }
  }
}

#[derive(Debug)]
pub struct VerificationMetadata {
  pub repo: String,
  pub remote: String,
  pub branch: String,
  pub commit: String,
}

// pub async fn get_job_artifacts(
//   project_slug: &str,
//   job_number: &str,
//   api_key: &str,
// ) -> Result<HashMap<String, String>, String> {
//   let mut headers = HeaderMap::new();
//   let mut api_key_header_value = HeaderValue::from_str(api_key).map_err(|e| e.to_string())?;
//   api_key_header_value.set_sensitive(true);
//   headers.insert("Circle-Token", api_key_header_value);
//   let client = Client::builder()
//     .default_headers(headers)
//     .build()
//     .map_err(|e| e.to_string())?;

//   let json = client
//     .get(format!(
//       "https://circleci.com/api/v2/project/{project_slug}/{job_number}/artifacts"
//     ))
//     .send()
//     .await
//     .map_err(|e| e.to_string())?
//     .json::<serde_json::Value>()
//     .await
//     .map_err(|e| e.to_string())?;

//   json
//     .as_object()
//     .and_then(|json_obj| json_obj.get("items"))
//     .and_then(|items_value| items_value.as_array())
//     .map(|items_arr| {
//       items_arr
//         .iter()
//         .flat_map(|item_value| {
//           item_value.as_object().and_then(|item_obj| {
//             let path = item_obj
//               .get("path")
//               .and_then(|path_value| path_value.as_str());
//             let url = item_obj.get("url").and_then(|url_value| url_value.as_str());
//             path
//               .zip(url)
//               .map(|(path, url)| (path.to_string(), url.to_string()))
//           })
//         })
//         .collect::<HashMap<String, String>>()
//     })
//     .ok_or("Failed to parse JSON".to_string())
// }
