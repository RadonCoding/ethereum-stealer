mod constants;
mod eth_wallet;
mod utils;
use colored::Colorize;
use eth_wallet::Wallet;
use std::{
    error::Error,
    io::{stdin, stdout, Read, Write},
    process,
    str::FromStr,
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    },
};
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

    let endpoint = constants::ENDPOINT;

    let amount_generated = Arc::new(AtomicI64::new(0));

    let thread_count = num_cpus::get();

    for i in 0..thread_count {
        let web3_con = eth_wallet::establish_web3_connection(&endpoint).await?;
        let amount_generated_clone = amount_generated.clone();

        tokio::spawn(async move {
            loop {
                WinConsole::set_title(
                    format!(
                        "Amount generated: {}",
                        amount_generated_clone.load(Ordering::Relaxed)
                    )
                    .as_str(),
                )
                .unwrap();

                match generate_eth(dst_addr, &web3_con).await {
                    Ok(()) => {
                        amount_generated_clone.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(err) => {
                        println!("Error: {}", err.to_string().bright_red());
                    }
                }
            }
        });

        println!("Started thread {}", i);
    }

    println!("Running {} threads press any key to stop...", thread_count);
    stdout().flush()?;
    stdin().read(&mut [0u8])?;

    // We use process::exit so it will kill all the threads
    process::exit(0);
}
