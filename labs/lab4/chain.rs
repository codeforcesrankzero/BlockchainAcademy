use serde::{Deserialize, Serialize};
use thiserror::Error;

use toychain::block::Block;
use toychain::crypto::{meets_difficulty, now_ts, Hash32};
use toychain::tx::Transaction;
use crate::state::State;

#[derive(Debug, Error)]
pub enum BlockError {
    #[error("prev hash mismatch")]    PrevHashMismatch,
    #[error("bad height")]            BadHeight,
    #[error("timestamp too old")]     BadTimestamp,
    #[error("pow not satisfied")]     BadPoW,
    #[error("tx invalid: {0}")]       TxInvalid(String),
    #[error("missing coinbase")]      NoCoinbase,
    #[error("invalid coinbase reward")] InvalidReward,
    #[error("multiple coinbase")]     MultipleCoinbase,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Chain {
    pub blocks:       Vec<Block>,
    pub difficulty:   u32,
    pub state:        State,
    pub block_reward: u64,
}

impl Chain {
    pub fn new(difficulty: u32, block_reward: u64) -> Self {
        Self { blocks: Vec::new(), difficulty, state: State::default(), block_reward }
    }

    pub fn genesis(&mut self) {
        todo!("создай пустой блок с height=0, заминируй его через block.mine(), добавь в self.blocks")
    }

    pub fn tip_hash(&self) -> Hash32 {
        todo!("хеш последнего блока: self.blocks.last().unwrap().header_hash()")
    }

    pub fn height(&self) -> u64 {
        todo!("self.blocks.last().map(|b| b.height).unwrap_or(0)")
    }

    pub fn add_block(&mut self, block: Block) -> Result<(), BlockError> {
        todo!("7 проверок из описания, потом self.state.apply_block + push")
    }

    pub fn mine_block(&self, txs: Vec<Transaction>, miner_addr: &str) -> Block {
        todo!("coinbase + txs, высота = tip.height+1, майн")
    }
}
