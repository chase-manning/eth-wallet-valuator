mod getTokens;
use getTokens::Token;
use reqwest::Error;
use std::env;

async fn get_balances(rpc_endpoint: &str) -> web3::Result<()> {
    let transport = web3::transports::Http::new(rpc_endpoint)?;
    let web3 = web3::Web3::new(transport);

    let mut count = 0;
    let account = "0x26EE5302D8cc0422EE5DCdF19668c663e2fAfb8E"
        .parse()
        .unwrap();

    while count < 10 {
        let balance = web3.eth().balance(account, None).await?;
        println!("Balance of {:?}: {}", account, balance);
        count += 1;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let rpc_endpoint = &args[1];

    let tokens: Vec<Token> = getTokens::get_tokens().await?;

    get_balances(rpc_endpoint).await.unwrap();

    Ok(())
}
