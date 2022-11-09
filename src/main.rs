mod get_tokens;
use get_tokens::Token;
use reqwest::Error;
use std::env;
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

async fn get_token_balance(
    rpc_endpoint: &str,
    address: &str,
    token_address: &str,
) -> web3::Result<f64> {
    let transport = Http::new(rpc_endpoint)?;
    let web3 = Web3::new(transport);
    let json = include_bytes!("abis/erc20.abi");
    let contract = Contract::from_json(web3.eth(), token_address.parse().unwrap(), json).unwrap();

    let balance = get_balance(&contract, address).await?;
    let decimals: U256 = get_decimals(&contract).await?;
    Ok(bn_to_float(balance, decimals.low_u64() as i32))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("Querying Balances");
    println!("====================================");
    let args: Vec<String> = env::args().collect();
    let account = &args[1];
    let rpc_endpoint = &args[2];
    let mut total_balance = 0.0;

    // Getting token data
    let tokens: Vec<Token> = get_tokens::get_tokens().await?;

    // Getting ETH Balance
    let eth_balance = get_eth_balance(rpc_endpoint, account).await.unwrap();
    let eth_price = tokens.iter().find(|t| t.symbol == "ETH").unwrap().price;
    let eth_balance = eth_balance * eth_price;
    println!("ETH: ${}", eth_balance);
    total_balance += eth_balance;

    // Getting ERC20 Token Balances
    for token in tokens {
        let token_balance = get_token_balance(rpc_endpoint, account, &token.address)
            .await
            .unwrap();
        let token_balance = token_balance * token.price;
        if token_balance > 0.0 {
            println!("{}: ${}", token.symbol.to_uppercase(), token_balance);
            total_balance += token_balance;
        }
    }

    // Printing total
    println!("Total Balance: ${}", total_balance);
    Ok(())
}
