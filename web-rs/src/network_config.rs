use std::fs;

use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkConfig {
    pub network_id: String,
    pub node_url: String,
    pub archival_url: String,
    pub wallet_url: String,
    pub helper_url: String,
    pub explorer_url: String,
}

pub fn load(path: &str) -> NetworkConfig {
    let handle = fs::File::open(path).expect(&format!("FATAL: Could not load network config path: {path}"));
    let reader = std::io::BufReader::new(handle);

    serde_json::from_reader(reader).expect("FATAL: Could not parse network config file")
}
