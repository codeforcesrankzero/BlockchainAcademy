use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use toychain::tx::{Transaction, TxError};
use toychain::block::Block;

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct State {
    pub balances: HashMap<String, u128>,
    pub nonces:   HashMap<String, u64>,
}

impl State {
    pub fn balance_of(&self, addr: &str) -> u128 {
        todo!("self.balances.get(addr).copied().unwrap_or(0)")
    }

    pub fn nonce_of(&self, addr: &str) -> u64 {
        todo!("self.nonces.get(addr).copied().unwrap_or(0)")
    }

    pub fn apply_tx(&mut self, tx: &Transaction) -> Result<(), TxError> {
        todo!("реализуй по шагам из описания выше")
    }


    pub fn apply_block(&mut self, block: &Block) -> Result<(), TxError> {
        todo!("for tx in &block.txs → apply_tx(tx)?")
    }
}
