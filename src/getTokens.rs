use reqwest::Error;
use serde::Deserialize;

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

pub async fn get_tokens() -> Result<(), Error> {
    let endpoint = format!("https://api.coingecko.com/api/v3/coins/list?include_platform=true");
    let response = reqwest::get(&endpoint).await?;
    let data: Vec<OptionCoin> = response.json().await?;
    println!("{:?}", data.len());

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

    println!("{:?}", ethereum_coins.len());
    println!("{:?}", ethereum_coins[0].ethereum);

    Ok(())
}
