use serde::{Deserialize, Serialize};
use reqwest::Client;
use tokio::time::{sleep, Duration};
use debug_macro::FxSpecialParser;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct RaydiumResponse {
    id: String,
    success: bool,
    pub data: HashMap<String, String>,
}


pub async fn fetch_rydm(client: &Client, address: &str) -> Result<RaydiumResponse, reqwest::Error> {
    let url = format!("https://api-v3.raydium.io/mint/price?mints={}", address);
    let res = client
        .get(&url)
        .send()
        .await?
        .json::<RaydiumResponse>()
        .await?;
    Ok(res)
}