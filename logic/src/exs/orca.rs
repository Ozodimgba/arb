use std::error::Error;

use serde::{Deserialize, Serialize};
use reqwest::Client;
use tokio::time::{sleep, Duration};
use debug_macro::FxSpecialParser;

#[derive(FxSpecialParser, Serialize, Deserialize)]
pub struct TokenInfo {
    mint: String,
    symbol: String,
    pub name: String,
    decimals: u32,
    logoURI: Option<String>,
    coingeckoId: Option<String>,
    whitelisted: bool,
    poolToken: bool,
}

#[derive(FxSpecialParser, Serialize, Deserialize)]
struct Volume {
    day: Option<f64>,
    week: Option<f64>,
    month: Option<f64>,
}

#[derive(FxSpecialParser, Serialize, Deserialize)]
struct PriceRange {
    min: Option<f64>,
    max: Option<f64>,
}

#[derive(FxSpecialParser, Serialize, Deserialize)]
struct PriceRanges {
    day: PriceRange,
    week: PriceRange,
    month: PriceRange,
}

#[derive(FxSpecialParser, Serialize, Deserialize)]
struct APR {
    day: Option<f64>,
    week: Option<f64>,
    month: Option<f64>,
}

#[derive(FxSpecialParser, Serialize, Deserialize)]
pub struct Whirlpool {
    address: String,
    pub tokenA: TokenInfo,
    tokenB: TokenInfo,
    whitelisted: bool,
    tickSpacing: u32,
    pub price: Option<f64>,
    lpFeeRate: Option<f64>,
    protocolFeeRate: Option<f64>,
    whirlpoolsConfig: String,
    modifiedTimeMs: Option<u64>,
    tvl: Option<f64>,
    volume: Option<Volume>,
    volumeDenominatedA: Option<Volume>,
    volumeDenominatedB: Option<Volume>,
    priceRange: Option<PriceRanges>,
    feeApr: Option<APR>,
    reward0Apr: Option<APR>,
    reward1Apr: Option<APR>,
    reward2Apr: Option<APR>,
    totalApr: Option<APR>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhirlpoolsData {
    whirlpools: Vec<Whirlpool>,
}

pub async fn fetch_raw_json(client: &Client) -> Result<String, reqwest::Error> {
    let res = client
        .get("https://api.mainnet.orca.so/v1/whirlpool/list")
        .send()
        .await?
        .text()
        .await?;
    Ok(res)
}

pub async fn fetch_orca_pools(client: &Client) -> Result<WhirlpoolsData, reqwest::Error> {
    let res = client
        .get("https://api.mainnet.orca.so/v1/whirlpool/list")
        .send()
        .await?
        .json::<WhirlpoolsData>()
        .await?;
    Ok(res)
}

pub async fn get_token_b_mints(client: &Client) -> Result<Vec<String>, Box<dyn Error>> {
    let whirlpools_data = fetch_orca_pools(client).await?;
    let tokens: Vec<String> = whirlpools_data
        .whirlpools
        .into_iter()
        .map(|whirlpool| whirlpool.tokenB.mint)
        .collect();
    Ok(tokens)
}

pub async fn filter_pairs(
    token_a_mint: Option<&str>,
    token_b_mint: Option<&str>,
    address: Option<&str>,
) -> Result<Option<Whirlpool>, Box<dyn std::error::Error>> {
    let client = Client::new();
    let pools: WhirlpoolsData = fetch_orca_pools(&client).await?;

    let res = pools.whirlpools.into_iter().find(|whirlpool| {
        if let Some(addr) = address {
            return &whirlpool.address == addr;
        }
        if let (Some(token_a), Some(token_b)) = (token_a_mint, token_b_mint) {
            return &whirlpool.tokenA.mint == token_a && &whirlpool.tokenB.mint == token_b;
        }
        false
    });

    Ok(res)
}


