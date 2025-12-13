use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;
use thiserror::Error;

use crate::block::Block;
use crate::chain::Chain;
use crate::state::State;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    #[error("File not found")]
    FileNotFound,
    #[error("Corrupted data")]
    CorruptedData,
}

pub struct Storage {
    data_dir: String,
}

impl Storage {
    pub fn new(data_dir: impl Into<String>) -> Self {
        Self {
            data_dir: data_dir.into(),
        }
    }

    pub fn save_chain(&self, chain: &Chain) -> Result<(), StorageError> {
        std::fs::create_dir_all(&self.data_dir)?;
        
        let blocks_path = format!("{}/blocks.dat", self.data_dir);
        let state_path = format!("{}/state.dat", self.data_dir);
        let meta_path = format!("{}/meta.dat", self.data_dir);
        
        let blocks_data = bincode::serialize(&chain.blocks)?;
        std::fs::write(&blocks_path, blocks_data)?;
        
        let state_data = bincode::serialize(&chain.state)?;
        std::fs::write(&state_path, state_data)?;
        
        #[derive(serde::Serialize)]
        struct Meta {
            difficulty: u32,
            block_reward: u64,
        }
        
        let meta = Meta {
            difficulty: chain.difficulty,
            block_reward: chain.block_reward,
        };
        let meta_data = bincode::serialize(&meta)?;
        std::fs::write(&meta_path, meta_data)?;
        
        Ok(())
    }

    pub fn load_chain(&self) -> Result<Chain, StorageError> {
        let blocks_path = format!("{}/blocks.dat", self.data_dir);
        let state_path = format!("{}/state.dat", self.data_dir);
        let meta_path = format!("{}/meta.dat", self.data_dir);
        
        if !Path::new(&blocks_path).exists() {
            return Err(StorageError::FileNotFound);
        }
        
        let blocks_data = std::fs::read(&blocks_path)?;
        let blocks: Vec<Block> = bincode::deserialize(&blocks_data)?;
        
        let state_data = std::fs::read(&state_path)?;
        let state: State = bincode::deserialize(&state_data)?;
        
        #[derive(serde::Deserialize)]
        struct Meta {
            difficulty: u32,
            block_reward: u64,
        }
        
        let meta_data = std::fs::read(&meta_path)?;
        let meta: Meta = bincode::deserialize(&meta_data)?;
        
        Ok(Chain {
            blocks,
            difficulty: meta.difficulty,
            state,
            block_reward: meta.block_reward,
        })
    }

    pub fn append_block(&self, block: &Block) -> Result<(), StorageError> {
        std::fs::create_dir_all(&self.data_dir)?;
        
        let blocks_path = format!("{}/blocks.dat", self.data_dir);
        
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&blocks_path)?;
        
        let block_data = bincode::serialize(block)?;
        let length = block_data.len() as u64;
        
        file.write_all(&length.to_le_bytes())?;
        file.write_all(&block_data)?;
        file.flush()?;
        
        Ok(())
    }

    pub fn load_blocks_incremental(&self) -> Result<Vec<Block>, StorageError> {
        let blocks_path = format!("{}/blocks.dat", self.data_dir);
        
        if !Path::new(&blocks_path).exists() {
            return Ok(Vec::new());
        }
        
        let mut file = File::open(&blocks_path)?;
        let mut blocks = Vec::new();
        
        loop {
            let mut length_buf = [0u8; 8];
            match file.read_exact(&mut length_buf) {
                Ok(_) => {},
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e.into()),
            }
            
            let length = u64::from_le_bytes(length_buf);
            let mut block_data = vec![0u8; length as usize];
            file.read_exact(&mut block_data)?;
            
            let block: Block = bincode::deserialize(&block_data)?;
            blocks.push(block);
        }
        
        Ok(blocks)
    }

    pub fn save_state(&self, state: &State) -> Result<(), StorageError> {
        std::fs::create_dir_all(&self.data_dir)?;
        
        let state_path = format!("{}/state.dat", self.data_dir);
        let state_data = bincode::serialize(state)?;
        std::fs::write(&state_path, state_data)?;
        
        Ok(())
    }

    pub fn chain_exists(&self) -> bool {
        let blocks_path = format!("{}/blocks.dat", self.data_dir);
        Path::new(&blocks_path).exists()
    }

    pub fn clear(&self) -> Result<(), StorageError> {
        let blocks_path = format!("{}/blocks.dat", self.data_dir);
        let state_path = format!("{}/state.dat", self.data_dir);
        let meta_path = format!("{}/meta.dat", self.data_dir);
        
        let _ = std::fs::remove_file(&blocks_path);
        let _ = std::fs::remove_file(&state_path);
        let _ = std::fs::remove_file(&meta_path);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::now_ts;

    #[test]
    fn test_save_load_chain() {
        let storage = Storage::new("test_data");
        storage.clear().unwrap();
        
        let mut chain = Chain::new(12, 50);
        chain.genesis();
        
        storage.save_chain(&chain).unwrap();
        
        let loaded_chain = storage.load_chain().unwrap();
        
        assert_eq!(chain.blocks.len(), loaded_chain.blocks.len());
        assert_eq!(chain.difficulty, loaded_chain.difficulty);
        assert_eq!(chain.block_reward, loaded_chain.block_reward);
        
        storage.clear().unwrap();
    }

    #[test]
    fn test_incremental_blocks() {
        let storage = Storage::new("test_data_inc");
        storage.clear().unwrap();
        
        let block1 = Block {
            height: 0,
            timestamp: now_ts(),
            prev_hash: [0u8; 32],
            nonce: 0,
            difficulty: 12,
            txs: vec![],
        };
        
        let block2 = Block {
            height: 1,
            timestamp: now_ts(),
            prev_hash: block1.header_hash(),
            nonce: 0,
            difficulty: 12,
            txs: vec![],
        };
        
        storage.append_block(&block1).unwrap();
        storage.append_block(&block2).unwrap();
        
        let blocks = storage.load_blocks_incremental().unwrap();
        
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].height, 0);
        assert_eq!(blocks[1].height, 1);
        
        storage.clear().unwrap();
    }
}

