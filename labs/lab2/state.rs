use std::collections::HashMap;
use toychain::tx::{Transaction, TxError};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct State {
    pub balances: HashMap<String, u128>,
    pub nonces: HashMap<String, u64>,
}

impl State {
    pub fn balance_of(&self, address: &str) -> u128 {
        todo!("Реализуйте получение баланса")
    }

    pub fn nonce_of(&self, address: &str) -> u64 {
        todo!("Реализуйте получение nonce")
    }

    pub fn apply_tx(&mut self, tx: &Transaction) -> Result<(), TxError> {
        todo!("Реализуйте применение транзакции")
    }

    pub fn apply_block(&mut self, block: &toychain::block::Block) -> Result<(), TxError> {
        todo!("Реализуйте применение блока")
    }
}
