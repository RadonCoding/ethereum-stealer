use super::utils;
use secp256k1::{rand::rngs::JitterRng, PublicKey, Secp256k1, SecretKey};
use std::error::Error;
use web3::{
    signing::{keccak256, SecretKeyRef},
    transports::{self, WebSocket},
    types::{Address, TransactionParameters, H256, U256},
    Web3,
};

pub struct Wallet {
    pub secret_key: SecretKey,
    pub public_key: PublicKey,
    pub public_address: Address,
}

impl Wallet {
    pub fn new(sk: &SecretKey, pk: &PublicKey) -> Self {
        let addr: Address = public_key_address(&pk);

        Wallet {
            secret_key: *sk,
            public_key: *pk,
            public_address: addr,
        }
    }

    pub async fn get_balance(
        &self,
        web3_connection: &Web3<WebSocket>,
    ) -> Result<U256, Box<dyn Error>> {
        let balance = web3_connection
            .eth()
            .balance(self.public_address, None)
            .await?;
        Ok(balance)
    }

    pub async fn sign_and_send(
        &self,
        web3_con: &Web3<transports::WebSocket>,
        transaction: TransactionParameters,
    ) -> Result<H256, Box<dyn Error>> {
        let signed = web3_con
            .accounts()
            .sign_transaction(transaction, SecretKeyRef::new(&self.secret_key))
            .await?;

        let transaction_result = web3_con
            .eth()
            .send_raw_transaction(signed.raw_transaction)
            .await?;
        Ok(transaction_result)
    }
}

pub fn generate_keypair() -> (SecretKey, PublicKey) {
    let secp = Secp256k1::new();
    let mut rng = JitterRng::new_with_timer(utils::get_nstime);
    secp.generate_keypair(&mut rng)
}

pub fn public_key_address(public_key: &PublicKey) -> Address {
    let public_key = public_key.serialize_uncompressed();
    let hash = keccak256(&public_key[1..]);
    Address::from_slice(&hash[12..])
}

pub fn create_eth_transaction(to: Address, wei_value: U256) -> TransactionParameters {
    TransactionParameters {
        to: Some(to),
        value: wei_value,
        ..Default::default()
    }
}

pub async fn establish_web3_connection(url: &str) -> Result<Web3<WebSocket>, Box<dyn Error>> {
    let transport = web3::transports::WebSocket::new(url).await?;
    Ok(web3::Web3::new(transport))
}
