use toychain::node::Node;
use toychain::network::Message;
use std::time::Duration;

#[tokio::main]
async fn main() {
    println!("P2P Network Demo\n");

    let mut node1 = Node::new("Node1", "127.0.0.1:8001", 12, 50);
    node1.start().await.expect("Failed to start node1");
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let mut node2 = Node::new("Node2", "127.0.0.1:8002", 12, 50);
    node2.start().await.expect("Failed to start node2");
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let mut node3 = Node::new("Node3", "127.0.0.1:8003", 12, 50);
    node3.start().await.expect("Failed to start node3");
    
    tokio::time::sleep(Duration::from_millis(100)).await;

    node2.connect_to("127.0.0.1:8001").await.expect("Failed to connect");
    node3.connect_to("127.0.0.1:8001").await.expect("Failed to connect");
    node3.connect_to("127.0.0.1:8002").await.expect("Failed to connect");
    
    tokio::time::sleep(Duration::from_millis(200)).await;

    println!("Peers: Node1={}, Node2={}, Node3={}", 
        node1.peer_count().await,
        node2.peer_count().await,
        node3.peer_count().await
    );

    println!("\nTest: Ping/Pong");
    node2.network.broadcast(Message::Ping).await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    node1.handle_all_messages().await.unwrap();

    println!("\nTest: Block Mining");
    node1.mine_block(10).await.unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    node2.handle_all_messages().await.unwrap();
    node3.handle_all_messages().await.unwrap();

    println!("\nTest: Transaction Broadcast");
    let tx = node1.create_transaction(&node2.address, 10);
    node1.broadcast_transaction(tx).await.unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    node2.handle_all_messages().await.unwrap();
    node3.handle_all_messages().await.unwrap();

    println!("\nTest: Chain Sync");
    node2.sync_chain().await;
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    node1.handle_all_messages().await.unwrap();
    node2.handle_all_messages().await.unwrap();

    println!("\nFinal: Node1 height={}, Node2 height={}, Node3 height={}", 
        node1.height(), node2.height(), node3.height());
}
