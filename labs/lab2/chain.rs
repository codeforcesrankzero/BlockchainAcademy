use thiserror::Error;
use serde::{Deserialize, Serialize};

use toychain::block::Block;
use toychain::crypto::{meets_difficulty, now_ts, Hash32};
use toychain::tx::Transaction;
use crate::state::State;

#[derive(Debug, Error)]
pub enum BlockError {
    #[error("prev hash mismatch")]
    PrevHashMismatch,
    #[error("bad height")]
    BadHeight,
    #[error("timestamp too old")]
    BadTimestamp,
    #[error("pow not satisfied")]
    BadPoW,
    #[error("tx invalid: {0}")]
    TxInvalid(String),
    #[error("missing coinbase")]
    NoCoinbase,
    #[error("invalid coinbase reward")]
    InvalidReward,
    #[error("multiple coinbase transactions")]
    MultipleCoinbase,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Chain {
    pub blocks: Vec<Block>,
    pub difficulty: u32,
    pub state: State,
    pub block_reward: u64,
}

impl Chain {
    pub fn new(difficulty: u32, block_reward: u64) -> Self {
        Self {
            blocks: Vec::new(),
            difficulty,
            state: State::default(),
            block_reward,
        }
    }

    pub fn genesis(&mut self) {
        todo!("Реализуйте создание genesis блока")
    }

    pub fn tip_hash(&self) -> Hash32 {
        todo!("Реализуйте получение tip_hash")
    }

    pub fn height(&self) -> u64 {
        todo!("Реализуйте получение высоты")
    }

    pub fn add_block(&mut self, block: Block) -> Result<(), BlockError> {
        todo!("Реализуйте добавление и валидацию блока")
    }

    pub fn mine_block(&self, txs: Vec<Transaction>, miner_address: &str) -> Block {
        todo!("Реализуйте майнинг блока")
    }
}
