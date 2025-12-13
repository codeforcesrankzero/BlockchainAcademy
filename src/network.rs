use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::block::Block;
use crate::tx::Transaction;

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Connection closed")]
    ConnectionClosed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    NewBlock(Block),
    NewTransaction(Transaction),
    GetBlocks { from_height: u64 },
    Blocks(Vec<Block>),
    GetHeaders { from_height: u64 },
    Headers(Vec<BlockHeader>),
    GetPeers,
    Peers(Vec<String>),
    Ping,
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub height: u64,
    pub hash: [u8; 32],
    pub prev_hash: [u8; 32],
    pub timestamp: i64,
}

pub struct Peer {
    address: String,
    stream: TcpStream,
}

impl Peer {
    fn new(address: String, stream: TcpStream) -> Self {
        Self { address, stream }
    }

    async fn send(&mut self, msg: &Message) -> Result<(), NetworkError> {
        let data = serde_json::to_vec(msg)
            .map_err(|e| NetworkError::Serialization(e.to_string()))?;
        
        let len = data.len() as u32;
        self.stream.write_all(&len.to_be_bytes()).await?;
        self.stream.write_all(&data).await?;
        self.stream.flush().await?;
        
        Ok(())
    }

    async fn receive(&mut self) -> Result<Message, NetworkError> {
        let mut len_buf = [0u8; 4];
        if self.stream.read_exact(&mut len_buf).await? == 0 {
            return Err(NetworkError::ConnectionClosed);
        }
        
        let len = u32::from_be_bytes(len_buf) as usize;
        let mut data = vec![0u8; len];
        self.stream.read_exact(&mut data).await?;
        
        let msg = serde_json::from_slice(&data)
            .map_err(|e| NetworkError::Serialization(e.to_string()))?;
        
        Ok(msg)
    }
}

pub struct Network {
    listen_addr: String,
    peers: Arc<RwLock<HashMap<String, mpsc::Sender<Message>>>>,
    incoming_tx: mpsc::Receiver<Message>,
    incoming_rx: mpsc::Sender<Message>,
    known_peers: Arc<RwLock<Vec<String>>>,
}

impl Network {
    pub fn new(listen_addr: impl Into<String>) -> Self {
        let (tx, rx) = mpsc::channel(100);
        
        Self {
            listen_addr: listen_addr.into(),
            peers: Arc::new(RwLock::new(HashMap::new())),
            incoming_tx: rx,
            incoming_rx: tx,
            known_peers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn start(&mut self) -> Result<(), NetworkError> {
        let listener = TcpListener::bind(&self.listen_addr).await?;
        let peers = self.peers.clone();
        let incoming_tx = self.incoming_rx.clone();
        
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        let peers_clone = peers.clone();
                        let incoming_tx = incoming_tx.clone();
                        let (tx, rx) = mpsc::channel(100);
                        let addr_str = addr.to_string();
                        
                        peers.write().await.insert(addr_str.clone(), tx);
                        
                        tokio::spawn(async move {
                            if let Err(e) = handle_connection(stream, addr_str, peers_clone, incoming_tx, rx).await {
                                eprintln!("Connection error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("Accept error: {}", e);
                    }
                }
            }
        });
        
        Ok(())
    }

    pub async fn connect(&mut self, addr: &str) -> Result<(), NetworkError> {
        let stream = TcpStream::connect(addr).await?;
        let (tx, rx) = mpsc::channel(100);
        
        self.peers.write().await.insert(addr.to_string(), tx);
        
        let peers = self.peers.clone();
        let incoming_tx = self.incoming_rx.clone();
        let addr = addr.to_string();
        
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, addr, peers, incoming_tx, rx).await {
                eprintln!("Connection error: {}", e);
            }
        });
        
        Ok(())
    }

    pub async fn broadcast(&self, msg: Message) {
        let peers = self.peers.read().await;
        for (addr, tx) in peers.iter() {
            if let Err(e) = tx.send(msg.clone()).await {
                eprintln!("Failed to send to {}: {}", addr, e);
            }
        }
    }

    pub async fn send_to(&self, addr: &str, msg: Message) -> Result<(), NetworkError> {
        let peers = self.peers.read().await;
        if let Some(tx) = peers.get(addr) {
            tx.send(msg).await
                .map_err(|_| NetworkError::ConnectionClosed)?;
            Ok(())
        } else {
            Err(NetworkError::ConnectionClosed)
        }
    }

    pub async fn receive(&mut self) -> Option<Message> {
        self.incoming_tx.recv().await
    }

    pub async fn peer_count(&self) -> usize {
        self.peers.read().await.len()
    }

    pub async fn get_peer_list(&self) -> Vec<String> {
        let peers = self.peers.read().await;
        peers.keys().cloned().collect()
    }

    pub async fn request_peers(&self) {
        self.broadcast(Message::GetPeers).await;
    }

    pub async fn discover_peers(&mut self, seed_nodes: Vec<String>) -> Result<(), NetworkError> {
        for seed in seed_nodes {
            if let Err(e) = self.connect(&seed).await {
                eprintln!("Failed to connect to seed node {}: {}", seed, e);
                continue;
            }
            
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            
            self.send_to(&seed, Message::GetPeers).await.ok();
        }
        
        Ok(())
    }

    pub async fn add_known_peer(&self, addr: String) {
        let mut known = self.known_peers.write().await;
        if !known.contains(&addr) && addr != self.listen_addr {
            known.push(addr);
        }
    }

    pub async fn connect_to_new_peers(&mut self, max_peers: usize) -> Result<(), NetworkError> {
        let current_count = self.peer_count().await;
        if current_count >= max_peers {
            return Ok(());
        }

        let known = self.known_peers.read().await.clone();
        let connected: Vec<String> = {
            let peers = self.peers.read().await;
            peers.keys().cloned().collect()
        };

        for addr in known {
            if current_count >= max_peers {
                break;
            }
            
            if !connected.contains(&addr) && addr != self.listen_addr {
                match self.connect(&addr).await {
                    Ok(_) => {
                        println!("Connected to discovered peer: {}", addr);
                        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                    }
                    Err(e) => {
                        eprintln!("Failed to connect to {}: {}", addr, e);
                    }
                }
            }
        }

        Ok(())
    }
}

async fn handle_connection(
    stream: TcpStream,
    addr: String,
    peers: Arc<RwLock<HashMap<String, mpsc::Sender<Message>>>>,
    incoming_tx: mpsc::Sender<Message>,
    mut outgoing_rx: mpsc::Receiver<Message>,
) -> Result<(), NetworkError> {
    let mut peer = Peer::new(addr.clone(), stream);
    
    loop {
        tokio::select! {
            result = peer.receive() => {
                match result {
                    Ok(msg) => {
                        if let Err(e) = incoming_tx.send(msg).await {
                            eprintln!("Failed to forward message: {}", e);
                            break;
                        }
                    }
                    Err(NetworkError::ConnectionClosed) => {
                        println!("Peer {} disconnected", addr);
                        break;
                    }
                    Err(e) => {
                        eprintln!("Error receiving from {}: {}", addr, e);
                        break;
                    }
                }
            }
            msg = outgoing_rx.recv() => {
                match msg {
                    Some(msg) => {
                        if let Err(e) = peer.send(&msg).await {
                            eprintln!("Failed to send to {}: {}", addr, e);
                            break;
                        }
                    }
                    None => {
                        println!("Outgoing channel closed for {}", addr);
                        break;
                    }
                }
            }
        }
    }
    
    peers.write().await.remove(&addr);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_connection() {
        let mut node1 = Network::new("127.0.0.1:8001");
        node1.start().await.unwrap();
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        let mut node2 = Network::new("127.0.0.1:8002");
        node2.start().await.unwrap();
        node2.connect("127.0.0.1:8001").await.unwrap();
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        let msg = Message::Ping;
        node2.broadcast(msg).await;
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        if let Some(received) = node1.receive().await {
            matches!(received, Message::Ping);
        }
    }
}

