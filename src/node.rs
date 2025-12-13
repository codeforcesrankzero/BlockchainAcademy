use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, timeout};
use thiserror::Error;

use crate::chain::{Chain, BlockError};
use crate::network::{Network, NetworkError, Message};
use crate::mempool::Mempool;
use crate::block::Block;
use crate::tx::{Transaction, TxError};
use crate::crypto::pubkey_to_address;

use ed25519_dalek::Keypair;
use rand::rngs::OsRng;

#[derive(Debug, Error)]
pub enum NodeError {
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
    #[error("Block error: {0}")]
    Block(#[from] BlockError),
    #[error("Transaction error: {0}")]
    Transaction(#[from] TxError),
}

pub struct Node {
    pub name: String,
    pub network: Network,
    pub chain: Chain,
    pub mempool: Arc<RwLock<Mempool>>,
    pub keypair: Keypair,
    pub address: String,
    running: Arc<RwLock<bool>>,
}

impl Node {
    pub fn new(
        name: impl Into<String>,
        listen_addr: impl Into<String>,
        difficulty: u32,
        block_reward: u64,
    ) -> Self {
        let mut rng = OsRng;
        let keypair = Keypair::generate(&mut rng);
        let address = pubkey_to_address(&keypair.public);
        
        let network = Network::new(listen_addr);
        let chain = Chain::new(difficulty, block_reward);
        let mempool = Arc::new(RwLock::new(Mempool::new()));
        
        Self {
            name: name.into(),
            network,
            chain,
            mempool,
            keypair,
            address,
            running: Arc::new(RwLock::new(false)),
        }
    }
    
    pub async fn start(&mut self) -> Result<(), NodeError> {
        self.network.start().await?;
        
        if self.chain.height() == 0 {
            self.chain.genesis();
        }
        
        *self.running.write().await = true;
        
        Ok(())
    }
    
    pub async fn stop(&mut self) {
        *self.running.write().await = false;
    }
    
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
    
    pub async fn connect_to(&mut self, addr: &str) -> Result<(), NodeError> {
        self.network.connect(addr).await?;
        Ok(())
    }
    
    pub async fn peer_count(&self) -> usize {
        self.network.peer_count().await
    }
    
    pub fn create_transaction(&self, to: &str, amount: u64) -> Transaction {
        let nonce = self.chain.state.nonce_of(&self.address);
        let mut tx = Transaction::new_unsigned(
            self.address.clone(),
            to.to_string(),
            amount,
            nonce,
            self.keypair.public.as_bytes().to_vec(),
        );
        tx.sign(&self.keypair);
        tx
    }
    
    pub async fn handle_message(&mut self, timeout_ms: u64) -> Result<bool, NodeError> {
        match timeout(
            Duration::from_millis(timeout_ms),
            self.network.receive()
        ).await {
            Ok(Some(msg)) => {
                self.process_message(msg).await?;
                Ok(true)
            }
            Ok(None) => Ok(false),
            Err(_) => Ok(false),
        }
    }
    
    pub async fn handle_all_messages(&mut self) -> Result<usize, NodeError> {
        let mut count = 0;
        while self.handle_message(10).await? {
            count += 1;
        }
        Ok(count)
    }
    
    pub async fn handle_messages(&mut self) {
        while let Ok(true) = self.handle_message(100).await {
        }
    }
    
    async fn process_message(&mut self, msg: Message) -> Result<(), NodeError> {
        match msg {
            Message::NewTransaction(tx) => {
                self.handle_new_transaction(tx).await?;
            }
            Message::NewBlock(block) => {
                self.handle_new_block(block).await?;
            }
            Message::GetBlocks { from_height } => {
                self.handle_get_blocks(from_height).await;
            }
            Message::Blocks(blocks) => {
                self.handle_blocks(blocks).await?;
            }
            Message::GetPeers => {
                self.handle_get_peers().await;
            }
            Message::Peers(addrs) => {
                self.handle_peers(addrs).await;
            }
            Message::Ping => {
                self.network.broadcast(Message::Pong).await;
            }
            Message::Pong => {
            }
            _ => {
            }
        }
        Ok(())
    }
    
    async fn handle_new_transaction(&mut self, tx: Transaction) -> Result<(), NodeError> {
        let tx_hash = tx.hash();
        
        {
            let mempool = self.mempool.read().await;
            if mempool.has_tx(&tx_hash) {
                return Ok(());
            }
        }
        
        let mut mempool = self.mempool.write().await;
        match mempool.add_tx(tx.clone(), &self.chain.state) {
            Ok(_) => {
                drop(mempool);
                self.network.broadcast(Message::NewTransaction(tx)).await;
            }
            Err(_e) => {
            }
        }
        
        Ok(())
    }
    
    async fn handle_new_block(&mut self, block: Block) -> Result<(), NodeError> {
        let block_height = block.height;
        
        match self.chain.add_block(block.clone()) {
            Ok(_) => {
                let mut mempool = self.mempool.write().await;
                for tx in &block.txs {
                    if !tx.is_coinbase() {
                        mempool.remove_by_nonce(&tx.from, tx.nonce);
                    }
                }
                drop(mempool);
                
                self.network.broadcast(Message::NewBlock(block)).await;
            }
            Err(_e) => {
                if block_height > self.chain.height() + 1 {
                    self.sync_chain().await;
                } else if block_height == self.chain.height() {
                    if let Some(our_block) = self.chain.get_block_at(block_height) {
                        if our_block.header_hash() != block.header_hash() {
                            self.sync_chain().await;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    async fn handle_get_blocks(&mut self, from_height: u64) {
        let blocks = self.chain.get_blocks_from(from_height);
        if !blocks.is_empty() {
            self.network.broadcast(Message::Blocks(blocks)).await;
        }
    }
    
    async fn handle_blocks(&mut self, blocks: Vec<Block>) -> Result<(), NodeError> {
        if blocks.is_empty() {
            return Ok(());
        }
        
        let last_height = blocks.last().unwrap().height;
        
        if self.chain.should_reorganize(last_height) {
            match self.chain.reorganize(blocks.clone()) {
                Ok(_) => {
                    let mut mempool = self.mempool.write().await;
                    for block in &blocks {
                        for tx in &block.txs {
                            if !tx.is_coinbase() {
                                mempool.remove_by_nonce(&tx.from, tx.nonce);
                            }
                        }
                    }
                }
                Err(_e) => {
                }
            }
        } else {
            for block in blocks {
                if block.height == self.chain.height() + 1 {
                    if let Ok(_) = self.chain.add_block(block.clone()) {
                        let mut mempool = self.mempool.write().await;
                        for tx in &block.txs {
                            if !tx.is_coinbase() {
                                mempool.remove_by_nonce(&tx.from, tx.nonce);
                            }
                        }
                    }
                } else if block.height > self.chain.height() + 1 {
                    self.sync_chain().await;
                    break;
                }
            }
        }
        Ok(())
    }
    
    async fn handle_get_peers(&self) {
        let peers = self.network.get_peer_list().await;
        self.network.broadcast(Message::Peers(peers)).await;
    }
    
    async fn handle_peers(&self, addrs: Vec<String>) {
        for addr in addrs {
            self.network.add_known_peer(addr).await;
        }
    }
    
    pub async fn sync_chain(&self) {
        self.network.broadcast(Message::GetBlocks { 
            from_height: self.chain.height() + 1 
        }).await;
    }
    
    pub async fn mine_block(&mut self, max_txs: usize) -> Result<Block, NodeError> {
        let txs = {
            let mempool = self.mempool.read().await;
            mempool.select_txs(&self.chain.state, max_txs)
        };
        
        let block = self.chain.mine_block(txs, &self.address);
        
        self.chain.add_block(block.clone())?;
        
        {
            let mut mempool = self.mempool.write().await;
            for tx in &block.txs {
                if !tx.is_coinbase() {
                    mempool.remove_by_nonce(&tx.from, tx.nonce);
                }
            }
        }
        
        self.network.broadcast(Message::NewBlock(block.clone())).await;
        
        Ok(block)
    }
    
    pub async fn broadcast_transaction(&mut self, tx: Transaction) -> Result<(), NodeError> {
        {
            let mut mempool = self.mempool.write().await;
            mempool.add_tx(tx.clone(), &self.chain.state)?;
        }
        
        self.network.broadcast(Message::NewTransaction(tx)).await;
        
        Ok(())
    }
    
    pub async fn run(&mut self) -> Result<(), NodeError> {
        while self.is_running().await {
            self.handle_message(100).await?;
        }
        Ok(())
    }
    
    pub fn balance(&self) -> u128 {
        self.chain.state.balance_of(&self.address)
    }
    
    pub fn balance_of(&self, address: &str) -> u128 {
        self.chain.state.balance_of(address)
    }
    
    pub fn height(&self) -> u64 {
        self.chain.height()
    }
    
    pub async fn mempool_size(&self) -> usize {
        self.mempool.read().await.len()
    }
}
