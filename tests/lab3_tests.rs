#[path = "../labs/lab3/simple_network.rs"]
mod simple_network;

use simple_network::{SimpleNetwork, Message};
use toychain::tx::Transaction;
use tokio::time::{sleep, Duration};

// ============================================
// TESTS FOR BASIC NETWORKING
// ============================================

#[tokio::test]
async fn test_network_creation() {
    let network = SimpleNetwork::new("127.0.0.1:7001");
    assert_eq!(network.peer_count().await, 0);
}

#[tokio::test]
async fn test_network_start() {
    let mut network = SimpleNetwork::new("127.0.0.1:7002");
    assert!(network.start().await.is_ok());
    
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_peer_connection() {
    let mut node1 = SimpleNetwork::new("127.0.0.1:7003");
    node1.start().await.unwrap();
    
    sleep(Duration::from_millis(100)).await;
    
    let mut node2 = SimpleNetwork::new("127.0.0.1:7004");
    node2.start().await.unwrap();
    node2.connect("127.0.0.1:7003").await.unwrap();
    
    sleep(Duration::from_millis(200)).await;
    
    assert!(node1.peer_count().await > 0 || node2.peer_count().await > 0);
}

#[tokio::test]
async fn test_ping_pong() {
    let mut node1 = SimpleNetwork::new("127.0.0.1:7005");
    node1.start().await.unwrap();
    
    sleep(Duration::from_millis(100)).await;
    
    let mut node2 = SimpleNetwork::new("127.0.0.1:7006");
    node2.start().await.unwrap();
    node2.connect("127.0.0.1:7005").await.unwrap();
    
    sleep(Duration::from_millis(200)).await;
    
    node2.broadcast(Message::Ping).await;
    
    sleep(Duration::from_millis(100)).await;
    
    if let Some(msg) = node1.receive().await {
        match msg {
            Message::Ping => {
                // OK
            }
            _ => panic!("Expected Ping message"),
        }
    }
}

#[tokio::test]
async fn test_transaction_broadcast() {
    let mut node1 = SimpleNetwork::new("127.0.0.1:7007");
    node1.start().await.unwrap();
    
    sleep(Duration::from_millis(100)).await;
    
    let mut node2 = SimpleNetwork::new("127.0.0.1:7008");
    node2.start().await.unwrap();
    node2.connect("127.0.0.1:7007").await.unwrap();
    
    sleep(Duration::from_millis(200)).await;
    
    let tx = Transaction::new_coinbase("test".to_string(), 50);
    node2.broadcast(Message::NewTransaction(tx.clone())).await;
    
    sleep(Duration::from_millis(100)).await;
    
    if let Some(msg) = node1.receive().await {
        match msg {
            Message::NewTransaction(received_tx) => {
                assert_eq!(received_tx.to, tx.to);
                assert_eq!(received_tx.amount, tx.amount);
            }
            _ => panic!("Expected NewTransaction message"),
        }
    }
}

