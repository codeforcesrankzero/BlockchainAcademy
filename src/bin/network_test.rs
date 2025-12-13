use toychain::node::Node;
use std::time::Duration;

#[tokio::main]
async fn main() {
    println!("P2P Integration Tests\n");

    let mut node1 = Node::new("Node1", "127.0.0.1:9001", 10, 50);
    let mut node2 = Node::new("Node2", "127.0.0.1:9002", 10, 50);
    let mut node3 = Node::new("Node3", "127.0.0.1:9003", 10, 50);
    let mut node4 = Node::new("Node4", "127.0.0.1:9004", 10, 50);
    
    node1.start().await.unwrap();
    node2.start().await.unwrap();
    node3.start().await.unwrap();
    node4.start().await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(200)).await;

    node2.connect_to("127.0.0.1:9001").await.unwrap();
    node3.connect_to("127.0.0.1:9001").await.unwrap();
    node3.connect_to("127.0.0.1:9002").await.unwrap();
    node4.connect_to("127.0.0.1:9001").await.unwrap();
    node4.connect_to("127.0.0.1:9002").await.unwrap();
    node4.connect_to("127.0.0.1:9003").await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(300)).await;
    
    println!("Peer counts: Node1={}, Node2={}, Node3={}, Node4={}", 
        node1.peer_count().await, node2.peer_count().await, 
        node3.peer_count().await, node4.peer_count().await);

    println!("\nTest: Block Propagation");
    node1.mine_block(0).await.unwrap();
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    let mut received_count = 0;
    for node in [&mut node2, &mut node3, &mut node4] {
        if node.handle_all_messages().await.unwrap() > 0 {
            if node.height() == 1 {
                received_count += 1;
            }
        }
    }
    
    println!("Block received by {}/3 nodes", received_count);
    println!("Heights: Node1={}, Node2={}, Node3={}, Node4={}", 
        node1.height(), node2.height(), node3.height(), node4.height());

    println!("\nTest: Transaction Broadcast");
    node2.mine_block(0).await.unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    node1.handle_all_messages().await.unwrap();
    node3.handle_all_messages().await.unwrap();
    node4.handle_all_messages().await.unwrap();
    
    let tx1 = node2.create_transaction(&node3.address, 10);
    let tx2 = node2.create_transaction(&node4.address, 10);
    
    node2.broadcast_transaction(tx1).await.unwrap();
    node2.broadcast_transaction(tx2).await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    for node in [&mut node1, &mut node3, &mut node4] {
        node.handle_all_messages().await.unwrap();
    }

    println!("\nTest: Concurrent Mining");
    node2.mine_block(10).await.unwrap();
    node3.mine_block(0).await.unwrap();
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    println!("Fork detected: Node2 height={}, Node3 height={}", 
        node2.height(), node3.height());

    println!("\nTest: Chain Synchronization");
    for _ in 0..3 {
        node2.mine_block(0).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        node1.handle_all_messages().await.unwrap();
        node3.handle_all_messages().await.unwrap();
        node4.handle_all_messages().await.unwrap();
    }
    
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    println!("\nFinal heights: Node1={}, Node2={}, Node3={}, Node4={}", 
        node1.height(), node2.height(), node3.height(), node4.height());
    
    println!("\nTests completed");
}
