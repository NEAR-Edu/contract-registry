use thiserror::Error;
use warp::reject::Reject;

#[derive(Error, Debug)]
pub enum CircleCiError {
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("JSON did not match expected schema")]
    JsonSchemaMismatch,
}

impl Reject for CircleCiError {}
