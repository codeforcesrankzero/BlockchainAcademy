use serde::{Deserialize, Serialize};

use crate::crypto::{sha256, meets_difficulty, Hash32};
use crate::tx::Transaction;

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
    fn txs_root(&self) -> Hash32 {
        let mut buf = Vec::with_capacity(self.txs.len() * 32);
        for tx in &self.txs {
            buf.extend_from_slice(&tx.hash());
        }
        sha256(&buf)
    }

    pub fn header_hash(&self) -> Hash32 {
        #[derive(Serialize)]
        struct HeaderView<'a> {
            height: u64,
            timestamp: i64,
            prev_hash: &'a Hash32,
            nonce: u64,
            difficulty: u32,
            txs_root: Hash32,
        }
        let view = HeaderView {
            height: self.height,
            timestamp: self.timestamp,
            prev_hash: &self.prev_hash,
            nonce: self.nonce,
            difficulty: self.difficulty,
            txs_root: self.txs_root(),
        };
        let bytes = bincode::serialize(&view).unwrap();
        sha256(&bytes)
    }

    pub fn mine(mut self) -> Self {
        loop {
            let h = self.header_hash();
            if meets_difficulty(&h, self.difficulty) {
                break self;
            }
            self.nonce = self.nonce.wrapping_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::meets_difficulty;

    #[test]
    fn mine_produces_valid_pow() {
        let b = Block {
            height: 0,
            timestamp: 0,
            prev_hash: [0u8; 32],
            nonce: 0,
            difficulty: 8,
            txs: vec![],
        }.mine();

        let h = b.header_hash();
        assert!(meets_difficulty(&h, 8));
    }
}
