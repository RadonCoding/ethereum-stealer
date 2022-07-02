mod constants;
mod eth_wallet;
mod utils;
use colored::Colorize;
use eth_wallet::Wallet;
use std::{error::Error, ops::Add, str::FromStr, time::Duration};
use web3::{transports::WebSocket, types::Address, Web3};
use win32console::console::WinConsole;

async fn generate_eth(dst_addr: Address, web3_con: &Web3<WebSocket>) -> Result<(), Box<dyn Error>> {
    let (sk, pk) = eth_wallet::generate_keypair();
    let src_wallet = Wallet::new(&sk, &pk);

    let block_number = web3_con.eth().block_number().await?;

    let wei_balance = src_wallet.get_balance(&web3_con).await?;
    let eth_balance = utils::wei_to_eth(wei_balance);

    if constants::ENABLE_LOG {
        println!("Address: {:?}", src_wallet.public_address);
        println!("Public key: {:?}", src_wallet.public_key);
        println!("Secret key: {:?}", src_wallet.secret_key);
        println!("Block number: {}", &block_number);
        println!("Wallet balance: {} ETH", eth_balance);
    }

    let mut transaction = eth_wallet::create_eth_transaction(dst_addr, wei_balance);

    if let Some(real_balance) = utils::estimate_gas(web3_con, &transaction).await {
        println!("{}", "Valid wallet".bright_green());

        transaction.value = real_balance;

        let transact_hash = src_wallet.sign_and_send(&web3_con, transaction).await?;

        println!("Transaction hash: {:?}", transact_hash);
    } else if constants::ENABLE_LOG {
        println!("{}", "Invalid wallet".bright_red());
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let dst_addr = Address::from_str(constants::ETHEREUM_ADDRESS)?;

    let endpoint = format!(
        "wss://mainnet.infura.io/ws/v3/{}",
        constants::INFURA_PROJECT_ID
    );

    let web3_con = eth_wallet::establish_web3_connection(&endpoint).await?;

    let mut amount_generated = 0;

    loop {
        WinConsole::set_title(format!("Amount generated: {}", amount_generated).as_str())?;
        generate_eth(dst_addr, &web3_con).await?;
        amount_generated = amount_generated.add(1);

        tokio::time::sleep(Duration::from_millis(constants::COOLDOWN)).await;
    }
}
