mod get_tokens;
use futures::StreamExt;
use get_tokens::Token;
use reqwest::Error;
use std::collections::HashMap;
use std::env;
use web3::contract::Contract;
use web3::transports::Http;
use web3::types::H160;
use web3::types::U256;
use web3::Web3;

fn bn_to_float(bn: U256, decimals: i32) -> f64 {
    return (bn.low_u64() as f64) / (10.0f64).powi(decimals);
}

async fn get_eth_balance(
    rpc_endpoint: &str,
    address: &str,
    tokens: &Vec<Token>,
) -> web3::Result<f64> {
    let transport = Http::new(rpc_endpoint)?;
    let web3 = Web3::new(transport);
    let account: H160 = address.parse().unwrap();
    let balance: U256 = web3.eth().balance(account, None).await?;
    let eth_price = tokens.iter().find(|t| t.symbol == "ETH").unwrap().price;
    let eth_balance = bn_to_float(balance, 18) * eth_price;
    println!("- ETH: ${:.2}", eth_balance);
    Ok(eth_balance)
}

async fn get_decimal(rpc_endpoint: &str, token: &Token) -> web3::Result<U256> {
    let transport = Http::new(rpc_endpoint)?;
    let web3 = Web3::new(transport);
    let json = include_bytes!("abis/erc20.abi");
    let contract = Contract::from_json(web3.eth(), token.address.parse().unwrap(), json).unwrap();
    let decimals: U256 = match contract
        .query(
            "decimals",
            (),
            None,
            web3::contract::Options::default(),
            None,
        )
        .await
    {
        Ok(decimals) => decimals,
        _ => U256::from(18),
    };
    Ok(decimals)
}

async fn get_balance(contract: &Contract<Http>, address: &str) -> web3::Result<U256> {
    let account: H160 = address.parse().unwrap();
    let balance: U256 = match contract
        .query(
            "balanceOf",
            account,
            None,
            web3::contract::Options::default(),
            None,
        )
        .await
    {
        Ok(balance) => balance,
        _ => {
            // println!("Error getting balance for {}", address);
            U256::from(0)
        }
    };

    Ok(balance)
}

async fn get_token_balance(
    rpc_endpoint: &str,
    address: &str,
    token: &Token,
    decimals: &HashMap<String, i32>,
) -> web3::Result<f64> {
    let transport = Http::new(rpc_endpoint)?;
    let web3 = Web3::new(transport);
    let json = include_bytes!("abis/erc20.abi");
    let contract = Contract::from_json(web3.eth(), token.address.parse().unwrap(), json).unwrap();

    let balance = get_balance(&contract, address).await?;
    let decimals = decimals.get(&token.address).unwrap();
    let scaled = bn_to_float(balance, *decimals);
    let value = scaled * token.price;
    if value > 0.0 {
        println!("- {}: ${:.2}", token.symbol.to_uppercase(), value);
    }
    Ok(value)
}

async fn get_token_balances(
    rpc_endpoint: &str,
    address: &str,
    tokens: &Vec<Token>,
    decimals: &HashMap<String, i32>,
) -> Result<f64, Error> {
    let futures = tokens.into_iter().map(|token| async move {
        get_token_balance(rpc_endpoint, address, &token, &decimals).await
    });

    let stream = futures::stream::iter(futures).buffer_unordered(16);

    let balances = stream.collect::<Vec<_>>().await;

    let mut total_balance = 0.0;
    for balance in balances {
        match balance {
            Ok(balance) => total_balance += balance,
            _ => {}
        }
    }

    Ok(total_balance)
}

async fn get_decimals(
    rpc_endpoint: &str,
    tokens: &Vec<Token>,
) -> Result<HashMap<String, i32>, Error> {
    println!("Getting decimals...");
    let decimals = {
        let text = std::fs::read_to_string("./decimal-cache.json").unwrap();
        serde_json::from_str::<HashMap<String, i32>>(&text).unwrap()
    };
    let decimal = |address: &str| -> i32 {
        match decimals.get(address) {
            Some(d) => *d,
            None => 69,
        }
    };

    let mut new_decimals: HashMap<String, i32> = HashMap::new();

    for token in tokens {
        let cached_decimal = decimal(&token.address);
        if cached_decimal == 69 {
            let decimals = get_decimal(&rpc_endpoint, &token).await.unwrap();
            new_decimals.insert(token.address.clone(), decimals.low_u64() as i32);
        } else {
            new_decimals.insert(token.address.clone(), cached_decimal.clone());
        }
    }

    let text = serde_json::to_string(&new_decimals).unwrap();
    std::fs::write("./decimal-cache.json", text).unwrap();

    println!("Got decimals!");
    Ok(new_decimals)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("");
    println!("Querying Balances");
    println!("====================================");

    let args: Vec<String> = env::args().collect();
    let account = &args[1];
    let rpc = &args[2];

    // Getting token data
    let tokens: Vec<Token> = get_tokens::get_tokens().await?;

    // Getting deciamls
    let decimals = get_decimals(rpc, &tokens).await?;

    // Getting ETH Balance
    println!("Getting wallet balance...");
    let mut total_balance = get_eth_balance(rpc, account, &tokens).await.unwrap();

    // Getting ERC20 Token Balances
    // Only dai tokens
    // let tokens = tokens
    //     .into_iter()
    //     .filter(|t| t.symbol == "dai")
    //     .collect::<Vec<Token>>();
    total_balance += get_token_balances(rpc, account, &tokens, &decimals).await?;

    // Printing total
    println!("====================================");
    println!("Total Balance: ${:.2}", total_balance);
    println!("====================================");
    Ok(())
}
