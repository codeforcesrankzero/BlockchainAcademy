use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{mpsc, RwLock};
use serde::{Serialize, Deserialize};
use thiserror::Error;

use toychain::block::Block;
use toychain::tx::Transaction;

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
    Ping,
    Pong,
}

pub struct SimpleNetwork {
    listen_addr: String,
    peers: Arc<RwLock<HashMap<String, mpsc::Sender<Message>>>>,
    incoming_rx: mpsc::Receiver<Message>,
    incoming_tx: mpsc::Sender<Message>,
}

impl SimpleNetwork {
    pub fn new(listen_addr: impl Into<String>) -> Self {
        todo!("Реализуйте создание сети")
    }

    pub async fn start(&mut self) -> Result<(), NetworkError> {
        todo!("Реализуйте запуск listener")
    }

    pub async fn connect(&mut self, addr: &str) -> Result<(), NetworkError> {
        todo!("Реализуйте подключение к peer")
    }

    pub async fn broadcast(&self, msg: Message) {
        todo!("Реализуйте broadcast")
    }

    pub async fn receive(&mut self) -> Option<Message> {
        todo!("Реализуйте получение сообщения")
    }

    pub async fn peer_count(&self) -> usize {
        self.peers.read().await.len()
    }
}

async fn send_message(stream: &mut TcpStream, msg: &Message) -> Result<(), NetworkError> {
    let data = serde_json::to_vec(msg)
        .map_err(|e| NetworkError::Serialization(e.to_string()))?;
    
    let len = data.len() as u32;
    stream.write_all(&len.to_be_bytes()).await?;
    stream.write_all(&data).await?;
    stream.flush().await?;
    
    Ok(())
}

async fn receive_message(stream: &mut TcpStream) -> Result<Message, NetworkError> {
    let mut len_buf = [0u8; 4];
    if stream.read_exact(&mut len_buf).await? == 0 {
        return Err(NetworkError::ConnectionClosed);
    }
    
    let len = u32::from_be_bytes(len_buf) as usize;
    let mut data = vec![0u8; len];
    stream.read_exact(&mut data).await?;
    
    let msg = serde_json::from_slice(&data)
        .map_err(|e| NetworkError::Serialization(e.to_string()))?;
    
    Ok(msg)
}
