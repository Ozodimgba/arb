use solana_sdk::signature::{Keypair, Signer};
use tokio::time::{sleep, Duration};
use tokio::task;
use debug_macro::FxSpecialParser;
use std::collections::HashMap;
use std::fmt;
use std::thread;
use std::error::Error;
use reqwest::Client;
use serde::Deserialize;
mod exs;
mod helpers;
use helpers::types::PriceEntry;

#[derive(Deserialize)]
struct PriceResponse {
    #[serde(flatten)]
    pools: HashMap<String, PoolData>,
}

#[derive(Deserialize, Debug)]
struct PoolData {
    poolId: String,
    poolAccount: String,
    tokenAAmount: String,
    tokenBAmount: String,
    poolTokenSupply: String,
    apy: ApyData,
    volume: VolumeData,
}

#[derive(Deserialize, Debug)]
struct ApyData {
    day: String,
    week: String,
    month: String,
}

#[derive(Deserialize, Debug)]
struct VolumeData {
    day: String,
    week: String,
    month: String,
}

async fn fetch_orca_price(client: &Client) -> Result<HashMap<String, PoolData>, reqwest::Error> {
    let res = client
        .get("https://api.orca.so/allPools")
        .send()
        .await?
        .json::<PriceResponse>()
        .await?;
    Ok(res.pools)
}

// async fn fetch_raydium_price(client: &Client) -> Result<f64, reqwest::Error> {
//     let res = client
//         .get("https://api.raydium.io/pairs")
//         .send()
//         .await?
//         .json::<PriceResponse>()
//         .await?;
//     Ok(res.price)
// }

async fn check_arbitrage(address: &str) {
    let client = Client::new();

    loop {
        let (orca_price, rydm_price, jup_price) = get_prices(address).await;

        let prices = vec![
            PriceEntry { name: String::from("ORCA"), price: orca_price.unwrap_or(0.0) },
            PriceEntry { name: String::from("RAYDIUM"), price: rydm_price.unwrap_or(0.0) },
            PriceEntry { name: String::from("JUPITER"), price: jup_price.unwrap_or(0.0) },
        ];

        let mut seg_tree = helpers::segment_tree::SegmentTree::new(prices);

        // Query minimum price in range [1, 4)
        let min_price = seg_tree.range_min_query(0, 3).unwrap();
       // println!("Minimum price in range: {:?}", min_price);

        // Query maximum price in range [1, 4)
        let max_price = seg_tree.range_max_query(0, 3).unwrap();
        //println!("Maximum price in range: {:?}", max_price);

        // Identify arbitrage opportunity
        if max_price.price > min_price.price {
            let price_difference = max_price.price - min_price.price;
            println!(
                "Max Arbitrage opportunity detected: buy {} in {} at {} and sell on {} at {}. Profit: ${}",
               address, min_price.name, min_price.price, max_price.name, max_price.price, price_difference
            );
        } else {
            println!("No arbitrage opportunity detected");
        }

        sleep(Duration::from_secs(1)).await;    
    }
}

#[derive(Debug)]
struct CustomError(String);

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for CustomError {}

async fn fetch_orca_market_by_address(address: &str) -> Result<Option<f64>, Box<dyn Error + Send + Sync>> {
    // "Czfq3xZZDmsdGdUyrNLtRhGc47cXcZtLG4crryfu44zE";
    
    match exs::orca::filter_pairs(Some(address), Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), None).await {
        Ok(Some(whirlpool)) => {
            if let Some(price) = whirlpool.price {
                Ok(Some(price))
            } else {
                println!("Whirlpool found, but no price available for address: {}", address);
                Ok(None)
            }
        }
        Ok(None) => {
            println!("No whirlpool found with address: {}", address);
            Ok(None)
        }
        Err(e) => {
            println!("Error fetching whirlpool: {}", e);
            Err(Box::new(CustomError(format!("Error fetching whirlpool: {}", e))) as Box<dyn Error + Send + Sync>)
        }
    }
}

async fn fetch_rydm_market(address: &str) -> Result<Option<f64>, Box<dyn Error>> {
    let client = Client::new();
    
    match exs::raydium::fetch_rydm(&client, &address).await {
        Ok(rydm_prices) => {
            let data = rydm_prices.data;
            if let Some(price) = data.get(address) {
                match price.parse::<f64>() {
                    Ok(parsed_price) => Ok(Some(parsed_price)),
                    Err(_) => {
                        println!("Unable to parse string: {}", price);
                        Ok(None)
                    }
                }
            } else {
                println!("No entry found for key: {}", address);
                Ok(None)
            }
        }
        Err(err) => {
            eprintln!("Error fetching Raydium prices: {:?}", err);
            Err(Box::new(err))
        }
    }
}

async fn fetch_jup_market(address: &str) -> Result<Option<f64>, Box<dyn Error>> {
    let client = Client::new();
    
    match exs::jupiter::fetch_jup(&client, &address).await {
        Ok(jup_prices) => {
            let data = jup_prices.data;
            if let Some(token_info) = data.get(address) {
                Ok(Some(token_info.price))
            } else {
                println!("No entry found for key: {}", address);
                Ok(None)
            }
        }
        Err(err) => {
            eprintln!("Error fetching Jupiter prices: {:?}", err);
            Err(Box::new(err))
        }
    }
}

async fn get_prices(address: &str) -> (Option<f64>, Option<f64>, Option<f64>) {
    let orca_result = fetch_orca_market_by_address(address).await.ok().flatten();
    let rydm_result = fetch_rydm_market(address).await.ok().flatten();
    let jup_result = fetch_jup_market(address).await.ok().flatten();

    (orca_result, rydm_result, jup_result)
}



// async fn get_prices(address: &str) {
//     let orca_result = fetch_orca_market_by_address(address).await;
//     let rydm_result = fetch_rydm_market(address).await;
//     let jup_result = fetch_jup_market(address).await;

//     match orca_result {
//         Ok(Some(price)) => println!("Orca Market Price: {}", price),
//         Ok(None) => println!("Orca Market Price: No data found"),
//         Err(e) => println!("Failed to fetch Orca Market Price: {}", e),
//     }

//     match rydm_result {
//         Ok(Some(price)) => println!("Raydium Market Price: {}", price),
//         Ok(None) => println!("Raydium Market Price: No data found"),
//         Err(e) => println!("Failed to fetch Raydium Market Price: {}", e),
//     }

//     match jup_result {
//         Ok(Some(price)) => println!("Jupiter Market Price: {}", price),
//         Ok(None) => println!("Jupiter Market Price: No data found"),
//         Err(e) => println!("Failed to fetch Jupiter Market Price: {}", e),
//     }
// }



#[tokio::main]
async fn main() {
    

    // check_arbitrage().await;

    // let client = Client::new();
    // let mut mints = exs::orca::get_token_b_mints(&client).await;
    // print!("{:?}", mints)

    // JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN
    //check_arbitrage().await

    let addresses: Vec<&str> = vec![
        "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN",
        "So11111111111111111111111111111111111111112",
        "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
        "rndrizKT3MK1iimdxRdWabcF7Zg7AR5T4nud4EkHBof"
        // Add more addresses as needed
    ];

    let mut handles = vec![];

    for &address in &addresses {
        let handle = task::spawn(check_arbitrage(address));
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

