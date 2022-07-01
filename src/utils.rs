use std::time::{SystemTime, UNIX_EPOCH};
use web3::{
    transports::WebSocket,
    types::{CallRequest, TransactionParameters, U256},
    Web3,
};

pub fn get_nstime() -> u64 {
    let dur = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    dur.as_secs() << 30 | dur.subsec_nanos() as u64
}

pub fn wei_to_eth(wei_val: U256) -> f64 {
    let res = wei_val.as_u128() as f64;
    res / 1_000_000_000_000_000_000.0
}

pub async fn estimate_gas(
    web3_con: &Web3<WebSocket>,
    transaction: &TransactionParameters,
) -> Option<U256> {
    if !transaction.value.is_zero() {
        let estimated_gas = web3_con
            .eth()
            .estimate_gas(CallRequest::from(transaction.clone()), None)
            .await
            .ok()?;
        let gas_price = web3_con.eth().gas_price().await.ok()?;

        let gas_value = estimated_gas * gas_price;
        let real_balance = transaction.value - gas_value;

        if !real_balance.is_zero() {
            return Some(real_balance);
        }
    }
    None
}
