use reqwest::Error;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
struct OptionPlatform {
    ethereum: Option<String>,
}

#[derive(Deserialize, Debug)]
struct OptionCoin {
    id: String,
    symbol: String,
    name: String,
    platforms: Option<OptionPlatform>,
}

#[derive(Deserialize, Debug)]
struct Coin {
    id: String,
    symbol: String,
    name: String,
    ethereum: String,
}

#[derive(Deserialize, Debug)]
struct Price {
    id: String,
    current_price: Option<f64>,
}

async fn get_prices_page(page: i32) -> Result<Vec<Price>, Error> {
    let endpoint = format!("https://api.coingecko.com/api/v3/coins/markets?vs_currency=usd&category=ethereum-ecosystem&order=market_cap_desc&per_page=250&page={}&sparkline=false", page);
    let response = reqwest::get(&endpoint).await?;
    let prices: Vec<Price> = response.json().await?;
    Ok(prices)
}

async fn get_prices() -> Result<Vec<Price>, Error> {
    let mut prices: Vec<Price> = Vec::new();
    let mut page = 1;
    loop {
        let page_prices = get_prices_page(page).await?;
        if page_prices.len() == 0 {
            break;
        }
        prices.extend(page_prices);
        page += 1;
    }

    Ok(prices)
}

async fn get_coins() -> Result<Vec<Coin>, Error> {
    let endpoint = format!("https://api.coingecko.com/api/v3/coins/list?include_platform=true");
    let response = reqwest::get(&endpoint).await?;
    let data: Vec<OptionCoin> = response.json().await?;

    let mut ethereum_coins: Vec<Coin> = Vec::new();
    for coin in data {
        if let Some(platform) = coin.platforms {
            if let Some(ethereum) = platform.ethereum {
                if ethereum.len() > 0 {
                    ethereum_coins.push(Coin {
                        id: coin.id,
                        symbol: coin.symbol,
                        name: coin.name,
                        ethereum,
                    });
                }
            }
        }
    }

    Ok(ethereum_coins)
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Token {
    pub name: String,
    pub symbol: String,
    pub address: String,
    pub price: f64,
}

pub async fn get_tokens() -> Result<Vec<Token>, Error> {
    let cache = {
        let text = std::fs::read_to_string("./token-cache.json").unwrap();
        serde_json::from_str::<Vec<Token>>(&text).unwrap()
    };
    if cache.len() > 0 {
        return Ok(cache);
    }
    println!("Getting tokens...");

    let coins: Vec<Coin> = get_coins().await?;
    let prices: Vec<Price> = get_prices().await?;

    let mut tokens: Vec<Token> = Vec::new();
    let mut addresses: Vec<String> = Vec::new();
    for coin in coins {
        for price in &prices {
            if coin.id == price.id {
                if let Some(price) = price.current_price {
                    if !addresses.contains(&coin.ethereum) {
                        addresses.push(coin.ethereum.clone());
                        tokens.push(Token {
                            name: coin.name.clone(),
                            symbol: coin.symbol.clone(),
                            address: coin.ethereum.clone(),
                            price,
                        });
                    }
                }
            }
        }
    }

    // Adding ETH
    tokens.push(Token {
        name: "Ethereum".to_string(),
        symbol: "ETH".to_string(),
        address: "0x0000000000000000000000000000000000000000".to_string(),
        price: prices.first().unwrap().current_price.unwrap(),
    });

    std::fs::write(
        "./token-cache.json",
        serde_json::to_string_pretty(&tokens).unwrap(),
    )
    .unwrap();

    println!("Got {} tokens", tokens.len());
    Ok(tokens)
}
