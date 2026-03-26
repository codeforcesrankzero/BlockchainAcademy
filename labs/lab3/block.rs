use serde::{Deserialize, Serialize};
use toychain::crypto::{sha256, meets_difficulty, Hash32};
use toychain::tx::Transaction;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Block {
    pub height:     u64,
    pub timestamp:  u64,
    pub prev_hash:  Hash32,
    pub nonce:      u64,
    pub difficulty: u32,
    pub txs:        Vec<Transaction>,
}

impl Block {
    pub fn txs_root(&self) -> Hash32 {
        let mut root = [0u8; 32];
        for tx in &self.txs {
            let combined = [root, tx.hash()].concat();
            root = sha256(&combined);
        }
        root
    }

    pub fn header_hash(&self) -> Hash32 {
        #[derive(Serialize)]
        struct Header<'a> {
            height: u64, timestamp: u64, prev_hash: &'a Hash32,
            nonce: u64, difficulty: u32, txs_root: Hash32,
        }
        let h = Header {
            height: self.height, timestamp: self.timestamp,
            prev_hash: &self.prev_hash, nonce: self.nonce,
            difficulty: self.difficulty, txs_root: self.txs_root(),
        };
        sha256(&bincode::serialize(&h).unwrap())
    }

    pub fn mine(mut self) -> Block {
        loop {
            if meets_difficulty(&self.header_hash(), self.difficulty) {
                return self;
            }
            self.nonce += 1;
        }
    }
}
