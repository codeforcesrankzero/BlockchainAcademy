use std::collections::HashMap;
use thiserror::Error;

use toychain::crypto::Hash32;
use toychain::state::State;
use toychain::tx::Transaction;

#[derive(Debug, Error)]
pub enum MempoolError {
    #[error("invalid signature: {0}")]
    InvalidSignature(String),
    #[error("bad nonce: expected {expected}, got {got}")]
    BadNonce { expected: u64, got: u64 },
    #[error("insufficient funds: has {has}, needs {needs}")]
    InsufficientFunds { has: u128, needs: u128 },
    #[error("duplicate transaction")]
    Duplicate,
}

pub struct Mempool {
    txs: HashMap<Hash32, Transaction>,
}

impl Mempool {
    pub fn new() -> Self {
        Self { txs: HashMap::new() }
    }

    pub fn add_tx(&mut self, tx: Transaction, state: &State) -> Result<(), MempoolError> {
        todo!("4 проверки из описания, потом self.txs.insert(tx.hash(), tx)")
    }

    pub fn select_txs(&self, _state: &State, max_count: usize) -> Vec<Transaction> {
        todo!("collect → sort by nonce → truncate to max_count")
    }

    pub fn remove_txs(&mut self, hashes: &[Hash32]) {
        todo!("for hash in hashes → self.txs.remove(hash)")
    }

    pub fn len(&self) -> usize { self.txs.len() }
    pub fn is_empty(&self) -> bool { self.txs.is_empty() }
}
