mod get_tokens;
use get_tokens::Token;
use reqwest::Error;
use std::env;
use web3::contract::Contract;
use web3::transports::Http;
use web3::types::H160;
use web3::types::U256;
use web3::Web3;

use futures::future;

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

async fn get_decimals(contract: &Contract<Http>) -> web3::Result<U256> {
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
        _ => U256::from(0),
    };

    Ok(balance)
}

async fn get_token_balance(rpc_endpoint: &str, address: &str, token: &Token) -> web3::Result<f64> {
    let transport = Http::new(rpc_endpoint)?;
    let web3 = Web3::new(transport);
    let json = include_bytes!("abis/erc20.abi");
    let contract = Contract::from_json(web3.eth(), token.address.parse().unwrap(), json).unwrap();

    let balance = get_balance(&contract, address).await?;
    let decimals: U256 = get_decimals(&contract).await?;
    let scaled = bn_to_float(balance, decimals.low_u32() as i32);
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
) -> Result<f64, Error> {
    let bodies = future::join_all(
        tokens
            .into_iter()
            .map(|token| async move { get_token_balance(rpc_endpoint, address, &token).await }),
    )
    .await;

    let mut total_balance = 0.0;
    for b in bodies {
        match b {
            Ok(b) => total_balance += b,
            Err(e) => eprintln!("Got an error: {}", e),
        }
    }

    Ok(total_balance)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("");
    println!("Querying Balances");
    println!("====================================");

    let args: Vec<String> = env::args().collect();
    let account = &args[1];
    let rpc = &args[2];
    let mut total_balance = 0.0;

    // Getting token data
    let tokens: Vec<Token> = get_tokens::get_tokens().await?;

    // Getting ETH Balance
    total_balance += get_eth_balance(rpc, account, &tokens).await.unwrap();

    // Getting ERC20 Token Balances
    total_balance += get_token_balances(rpc, account, &tokens).await?;

    // Printing total
    println!("====================================");
    println!("Total Balance: ${:.2}", total_balance);
    println!("====================================");
    Ok(())
}
