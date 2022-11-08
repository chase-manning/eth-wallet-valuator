mod get_tokens;
use get_tokens::Token;
use reqwest::Error;
use std::env;
use std::fs;
use std::io;
use web3::contract::Contract;
use web3::transports::Http;
use web3::types::H160;
use web3::types::U256;
use web3::Web3;

fn bn_to_float(bn: U256, decimals: i32) -> f64 {
    return (bn.low_u64() as f64) / (10.0f64).powi(decimals);
}

async fn get_eth_balance(rpc_endpoint: &str, address: &str) -> web3::Result<f64> {
    let transport = Http::new(rpc_endpoint)?;
    let web3 = Web3::new(transport);
    let account: H160 = address.parse().unwrap();
    let balance: U256 = web3.eth().balance(account, None).await?;
    Ok(bn_to_float(balance, 18))
}

async fn get_token_balance(
    rpc_endpoint: &str,
    address: &str,
    token_address: &str,
) -> web3::Result<f64> {
    let transport = Http::new(rpc_endpoint)?;
    let web3 = Web3::new(transport);
    let account: H160 = address.parse().unwrap();
    let json = include_bytes!("abis/erc20.abi");
    let contract = Contract::from_json(web3.eth(), token_address.parse().unwrap(), json).unwrap();
    let balance: U256 = contract
        .query(
            "balanceOf",
            account,
            None,
            web3::contract::Options::default(),
            None,
        )
        .await
        .unwrap();
    let decimals: U256 = contract
        .query(
            "decimals",
            (),
            None,
            web3::contract::Options::default(),
            None,
        )
        .await
        .unwrap();
    Ok(bn_to_float(balance, decimals.low_u64() as i32))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let account = &args[1];
    let rpc_endpoint = &args[2];

    let tokens: Vec<Token> = get_tokens::get_tokens().await?;
    // let usdc = "0x79f16957A1a9dF84f50f6539cc13d8054E9E4143";
    let account = "0x79f16957A1a9dF84f50f6539cc13d8054E9E4143";
    let dai = "0x6B175474E89094C44Da98b954EedeAC495271d0F";

    let mut total_balance = 0.0;
    let eth_balance = get_eth_balance(rpc_endpoint, account).await.unwrap();
    let eth_price = tokens.iter().find(|t| t.symbol == "ETH").unwrap().price;
    let eth_balance = eth_balance * eth_price;
    println!("ETH Balance: {}", eth_balance);
    total_balance += eth_balance;

    let total_tokens = tokens.len();
    let mut count = 0;
    for token in tokens {
        let token_balance = get_token_balance(rpc_endpoint, account, &token.address)
            .await
            .unwrap();
        let token_balance = token_balance * token.price;
        println!("{} Balance: {}", token.symbol, token_balance);
        total_balance += token_balance;
        count += 1;
        println!("{}/{}", count, total_tokens);
    }

    Ok(())
}
