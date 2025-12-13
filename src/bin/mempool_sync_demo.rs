use toychain::node::Node;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    println!("Mempool Synchronization Demo\n");

    let mut node1 = Node::new("Node1", "127.0.0.1:7001", 12, 50);
    let mut node2 = Node::new("Node2", "127.0.0.1:7002", 12, 50);
    let mut node3 = Node::new("Node3", "127.0.0.1:7003", 12, 50);
    
    node1.start().await.unwrap();
    node2.start().await.unwrap();
    node3.start().await.unwrap();
    sleep(Duration::from_millis(100)).await;

    node1.connect_to("127.0.0.1:7002").await.unwrap();
    node1.connect_to("127.0.0.1:7003").await.unwrap();
    node2.connect_to("127.0.0.1:7003").await.unwrap();
    sleep(Duration::from_millis(200)).await;
    
    node2.sync_chain().await;
    node3.sync_chain().await;
    sleep(Duration::from_millis(200)).await;
    
    node1.handle_all_messages().await.unwrap();
    node2.handle_all_messages().await.unwrap();
    node3.handle_all_messages().await.unwrap();

    println!("Mining initial blocks...");
    for _ in 1..=3 {
        node1.mine_block(10).await.unwrap();
        sleep(Duration::from_millis(100)).await;
        
        node2.handle_all_messages().await.unwrap();
        node3.handle_all_messages().await.unwrap();
    }

    sleep(Duration::from_millis(200)).await;

    println!("\nChain heights: Node1={}, Node2={}, Node3={}", 
        node1.height(), node2.height(), node3.height());
    println!("Balances: Node1={}, Node2={}, Node3={}", 
        node1.balance(), node2.balance(), node3.balance());

    println!("\nBroadcasting transactions...");
    let tx1 = node1.create_transaction(&node2.address, 50);
    node1.broadcast_transaction(tx1).await.unwrap();
    sleep(Duration::from_millis(200)).await;

    node2.handle_all_messages().await.unwrap();
    node3.handle_all_messages().await.unwrap();

    let tx2 = node1.create_transaction(&node3.address, 30);
    node1.broadcast_transaction(tx2).await.unwrap();
    sleep(Duration::from_millis(200)).await;

    node2.handle_all_messages().await.unwrap();
    node3.handle_all_messages().await.unwrap();

    println!("Mempool sizes: Node1={}, Node2={}, Node3={}", 
        node1.mempool_size().await, node2.mempool_size().await, node3.mempool_size().await);

    println!("\nMining block with transactions...");
    node2.mine_block(10).await.unwrap();
    sleep(Duration::from_millis(200)).await;

    node1.handle_all_messages().await.unwrap();
    node3.handle_all_messages().await.unwrap();

    println!("\nFinal state:");
    println!("Heights: Node1={}, Node2={}, Node3={}", 
        node1.height(), node2.height(), node3.height());
    println!("Balances: Node1={}, Node2={}, Node3={}", 
        node1.balance(), node2.balance(), node3.balance());
    println!("Mempool sizes: Node1={}, Node2={}, Node3={}", 
        node1.mempool_size().await, node2.mempool_size().await, node3.mempool_size().await);

    let total = node1.balance() + node2.balance() + node3.balance();
    let expected = node1.height() * 50;
    println!("\nTotal supply: {} (expected: {})", total, expected);
}
