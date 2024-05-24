use serde::{Deserialize, Serialize};
use reqwest::Client;
use tokio::time::{sleep, Duration};
use debug_macro::FxSpecialParser;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenInfo {
    id: String,
    mintSymbol: String,
    vsToken: String,
    vsTokenSymbol: String,
    pub price: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JupResponse {
    pub data: HashMap<String, TokenInfo>,
    timeTaken: f64,
}

pub async fn fetch_jup(client: &Client, address: &str) -> Result<JupResponse, reqwest::Error> {
    let url = format!("https://price.jup.ag/v6/price?ids={}", address);
    let res = client
        .get(&url)
        .send()
        .await?
        .json::<JupResponse>()
        .await?;
    Ok(res)
}