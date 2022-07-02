mod constants;
mod eth_wallet;
mod utils;
use ansi_term::Colour::Blue;
use ansi_term::Colour::Green;
use ansi_term::Colour::Red;
use eth_wallet::Wallet;
use std::{
    error::Error,
    io::{stdin, stdout, Read, Write},
    str::FromStr,
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    },
    time::Duration,
};
use web3::{transports::WebSocket, types::Address, Web3};
use win32console::console::WinConsole;

async fn generate_eth(dst_addr: Address, web3_con: &Web3<WebSocket>) -> Result<(), Box<dyn Error>> {
    let (sk, pk) = eth_wallet::generate_keypair();
    let src_wallet = Wallet::new(&sk, &pk);

    let block_number = web3_con.eth().block_number().await?;

    let wei_balance = src_wallet.get_balance(&web3_con).await?;
    let eth_balance = utils::wei_to_eth(wei_balance);

    let mut transaction = eth_wallet::create_eth_transaction(dst_addr, wei_balance);

    if let Some(real_balance) = utils::estimate_gas(web3_con, &transaction).await {
        println!("{}", Green.paint("Valid wallet"));

        transaction.value = real_balance;

        let transact_hash = src_wallet.sign_and_send(&web3_con, transaction).await?;

        println!("Transaction hash: {:?}", transact_hash);
        println!("Press enter to continue...");

        stdout().flush()?;
        stdin().read(&mut [0u8])?;
    } else if constants::ENABLE_LOG {
        println!("{}", Red.paint("Invalid wallet"));
    }

    if constants::ENABLE_LOG {
        println!("Address: {:?}", src_wallet.public_address);
        println!("Public key: {:?}", src_wallet.public_key);
        println!("Secret key: {:?}", src_wallet.secret_key);
        println!("Block number: {}", &block_number);
        println!(
            "Wallet balance: {} ETH",
            Blue.paint(eth_balance.to_string())
        );
        println!();
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    match ansi_term::enable_ansi_support() {
        Ok(()) => (),
        Err(_) => {
            println!("Failed to enable ANSI support");
        }
    }

    let dst_addr = Address::from_str(constants::ETHEREUM_ADDRESS)?;

    let web3_con = eth_wallet::establish_web3_connection(&constants::ENDPOINT).await?;
    let amount_generated = Arc::new(AtomicI64::new(0));

    let thread_count = num_cpus::get();

    for i in 0..thread_count {
        let web3_con_clone = web3_con.clone();
        let amount_generated_clone = amount_generated.clone();

        tokio::task::spawn(async move {
            loop {
                WinConsole::set_title(
                    format!(
                        "Amount generated: {}",
                        amount_generated_clone.load(Ordering::Relaxed)
                    )
                    .as_str(),
                )
                .unwrap();

                match generate_eth(dst_addr, &web3_con_clone).await {
                    Ok(()) => {
                        amount_generated_clone.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(err) => {
                        println!("Error: {}", Red.paint(err.to_string()));
                    }
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        });

        println!("Started thread {}", i);
    }

    println!("Generating with {} threads", thread_count);

    println!("Press enter to stop...");
    stdout().flush()?;
    stdin().read(&mut [0u8])?;

    Ok(())
}
