use thiserror::Error;
use serde::{Deserialize, Serialize};

use crate::block::Block;
use crate::crypto::{meets_difficulty, now_ts, Hash32};
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
        Self { blocks: Vec::new(), difficulty, state: State::default(), block_reward }
    }

    pub fn genesis(&mut self) {
        let genesis = Block {
            height: 0,
            timestamp: now_ts(),
            prev_hash: [0u8; 32],
            nonce: 0,
            difficulty: self.difficulty,
            txs: vec![],
        };
        self.blocks.push(genesis.mine());
    }

    pub fn tip_hash(&self) -> Hash32 {
        self.blocks.last().unwrap().header_hash()
    }

    pub fn height(&self) -> u64 {
        self.blocks.last().map(|b| b.height).unwrap_or(0)
    }

    pub fn add_block(&mut self, block: Block) -> Result<(), BlockError> {
        let expected_height = self.blocks.last().unwrap().height + 1;
        if block.height != expected_height {
            return Err(BlockError::BadHeight);
        }
        if block.prev_hash != self.tip_hash() {
            return Err(BlockError::PrevHashMismatch);
        }
        if block.timestamp < self.blocks.last().unwrap().timestamp {
            return Err(BlockError::BadTimestamp);
        }
        if !meets_difficulty(&block.header_hash(), block.difficulty) {
            return Err(BlockError::BadPoW);
        }

        if block.height > 0 {
            if block.txs.is_empty() || !block.txs[0].is_coinbase() {
                return Err(BlockError::NoCoinbase);
            }
            if block.txs[0].amount != self.block_reward {
                return Err(BlockError::InvalidReward);
            }
            for tx in &block.txs[1..] {
                if tx.is_coinbase() {
                    return Err(BlockError::MultipleCoinbase);
                }
            }
        }

        let mut next_state = self.state.clone();
        if let Err(e) = next_state.apply_block(&block) {
            return Err(BlockError::TxInvalid(e.to_string()));
        }

        self.state = next_state;
        self.blocks.push(block);
        Ok(())
    }

    pub fn mine_block(&self, txs: Vec<crate::tx::Transaction>, miner_address: &str) -> Block {
        let mut all_txs = vec![crate::tx::Transaction::new_coinbase(
            miner_address.to_string(),
            self.block_reward,
        )];
        all_txs.extend(txs);

        Block {
            height: self.blocks.last().unwrap().height + 1,
            timestamp: now_ts(),
            prev_hash: self.tip_hash(),
            nonce: 0,
            difficulty: self.difficulty,
            txs: all_txs,
        }
        .mine()
    }

    pub fn get_blocks_from(&self, from_height: u64) -> Vec<Block> {
        self.blocks
            .iter()
            .filter(|b| b.height >= from_height)
            .cloned()
            .collect()
    }

    pub fn get_block_at(&self, height: u64) -> Option<&Block> {
        self.blocks.iter().find(|b| b.height == height)
    }

    pub fn find_fork_point(&self, other_blocks: &[Block]) -> Option<u64> {
        for block in other_blocks.iter().rev() {
            if let Some(our_block) = self.get_block_at(block.height) {
                if our_block.header_hash() == block.header_hash() {
                    return Some(block.height);
                }
            }
        }
        None
    }

    pub fn validate_chain(&self, blocks: &[Block]) -> Result<State, BlockError> {
        if blocks.is_empty() {
            return Ok(State::default());
        }

        let mut state = State::default();

        for i in 0..blocks.len() {
            let block = &blocks[i];

            if i > 0 {
                let prev = &blocks[i - 1];
                if block.height != prev.height + 1 {
                    return Err(BlockError::BadHeight);
                }
                if block.prev_hash != prev.header_hash() {
                    return Err(BlockError::PrevHashMismatch);
                }
                if block.timestamp < prev.timestamp {
                    return Err(BlockError::BadTimestamp);
                }
            }

            if !meets_difficulty(&block.header_hash(), block.difficulty) {
                return Err(BlockError::BadPoW);
            }

            if block.height > 0 {
                if block.txs.is_empty() || !block.txs[0].is_coinbase() {
                    return Err(BlockError::NoCoinbase);
                }
                if block.txs[0].amount != self.block_reward {
                    return Err(BlockError::InvalidReward);
                }
                for tx in &block.txs[1..] {
                    if tx.is_coinbase() {
                        return Err(BlockError::MultipleCoinbase);
                    }
                }
            }

            if let Err(e) = state.apply_block(block) {
                return Err(BlockError::TxInvalid(e.to_string()));
            }
        }

        Ok(state)
    }

    pub fn reorganize(&mut self, new_blocks: Vec<Block>) -> Result<(), BlockError> {
        if new_blocks.is_empty() {
            return Ok(());
        }

        let new_height = new_blocks.last().unwrap().height;
        
        if new_height <= self.height() {
            return Ok(());
        }

        let fork_height = if new_blocks[0].height == 0 {
            let mut fork_point = 0u64;
            for new_block in new_blocks.iter() {
                if let Some(our_block) = self.get_block_at(new_block.height) {
                    if our_block.header_hash() == new_block.header_hash() {
                        fork_point = new_block.height;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            fork_point
        } else {
            self.find_fork_point(&new_blocks).unwrap_or(0)
        };

        let new_state = self.validate_chain(&new_blocks)?;

        self.blocks = new_blocks;
        self.state = new_state;

        println!("Chain reorganization: fork at {}, new height {}", fork_height, self.height());

        Ok(())
    }

    pub fn should_reorganize(&self, other_height: u64) -> bool {
        other_height > self.height()
    }
}
