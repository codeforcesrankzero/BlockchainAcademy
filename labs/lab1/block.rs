use serde::{Deserialize, Serialize};

use toychain::crypto::{sha256, meets_difficulty, Hash32};

use crate::transaction::Transaction;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Block {
    pub height: u64,
    pub timestamp: i64,
    pub prev_hash: Hash32,
    pub nonce: u64,
    pub difficulty: u32,
    pub txs: Vec<Transaction>,
}

impl Block {
    pub fn txs_root(&self) -> Hash32 {
        todo!("Реализуйте вычисление txs_root")
    }

    pub fn header_hash(&self) -> Hash32 {
        todo!("Реализуйте вычисление header_hash")
    }

    pub fn mine(mut self) -> Self {
        todo!("Реализуйте майнинг блока")
    }
}
