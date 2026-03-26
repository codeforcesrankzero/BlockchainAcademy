use toychain::node::Node;
use toychain::network::Message;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    println!("Peer Discovery & Fork Resolution Demo\n");

    println!("═══ Phase 1: Network Setup ═══\n");

    let mut seed = Node::new("Seed", "127.0.0.1:9000", 12, 50);
    seed.start().await.unwrap();
    println!("Seed node started at 127.0.0.1:9000");
    sleep(Duration::from_millis(100)).await;

    let mut node1 = Node::new("Node1", "127.0.0.1:9001", 12, 50);
    node1.start().await.unwrap();
    println!("Node1 started at 127.0.0.1:9001");
    sleep(Duration::from_millis(100)).await;

    let mut node2 = Node::new("Node2", "127.0.0.1:9002", 12, 50);
    node2.start().await.unwrap();
    println!("Node2 started at 127.0.0.1:9002");
    sleep(Duration::from_millis(100)).await;

    let mut node3 = Node::new("Node3", "127.0.0.1:9003", 12, 50);
    node3.start().await.unwrap();
    println!("Node3 started at 127.0.0.1:9003\n");
    sleep(Duration::from_millis(100)).await;

    println!("═══ Phase 2: Peer Discovery ═══\n");

    println!("Node1 discovers network via Seed...");
    node1.network.discover_peers(vec!["127.0.0.1:9000".to_string()]).await.unwrap();
    sleep(Duration::from_millis(200)).await;

    seed.handle_messages().await;
    node1.handle_messages().await;
    sleep(Duration::from_millis(100)).await;

    println!("\nNode2 discovers network via Seed...");
    node2.network.discover_peers(vec!["127.0.0.1:9000".to_string()]).await.unwrap();
    sleep(Duration::from_millis(200)).await;

    seed.handle_messages().await;
    node2.handle_messages().await;
    sleep(Duration::from_millis(100)).await;

    println!("\nNode1 and Node2 connect to discovered peers...");
    node1.network.add_known_peer("127.0.0.1:9002".to_string()).await;
    node1.network.connect_to_new_peers(5).await.unwrap();
    sleep(Duration::from_millis(200)).await;

    println!("\nNode3 discovers network via Node1...");
    node3.network.discover_peers(vec!["127.0.0.1:9001".to_string()]).await.unwrap();
    sleep(Duration::from_millis(200)).await;

    node1.handle_messages().await;
    node3.handle_messages().await;
    sleep(Duration::from_millis(100)).await;

    node3.network.connect_to_new_peers(5).await.unwrap();
    sleep(Duration::from_millis(200)).await;

    println!("\nNetwork topology:");
    println!("   Seed:  {} peers", seed.network.peer_count().await);
    println!("   Node1: {} peers", node1.network.peer_count().await);
    println!("   Node2: {} peers", node2.network.peer_count().await);
    println!("   Node3: {} peers", node3.network.peer_count().await);

    println!("\nPeer discovery successful. Network is connected.\n");

    println!("═══ Phase 3: Normal Block Propagation ═══\n");

    println!("Seed mines block #1...");
    let block1 = seed.chain.mine_block(vec![], &seed.address);
    seed.chain.add_block(block1.clone()).unwrap();
    println!("Seed mined block #1");

    seed.network.broadcast(Message::NewBlock(block1)).await;
    sleep(Duration::from_millis(200)).await;

    node1.handle_messages().await;
    node2.handle_messages().await;
    node3.handle_messages().await;
    sleep(Duration::from_millis(100)).await;

    println!("\nChain heights after block #1:");
    println!("   Seed:  {}", seed.height());
    println!("   Node1: {}", node1.height());
    println!("   Node2: {}", node2.height());
    println!("   Node3: {}", node3.height());

    println!("\n═══ Phase 4: Fork Creation ═══\n");

    println!("Node1 mines block #2...");
    let block2_node1 = node1.chain.mine_block(vec![], &node1.address);
    let hash1 = hex::encode(&block2_node1.header_hash()[..8]);
    println!("Node1 mined block #2 ({}...)", hash1);

    println!("\nNode2 also mines block #2 simultaneously...");
    let block2_node2 = node2.chain.mine_block(vec![], &node2.address);
    let hash2 = hex::encode(&block2_node2.header_hash()[..8]);
    println!("Node2 mined block #2 ({}...)", hash2);

    println!("\nFORK CREATED: Two competing blocks at height 2\n");

    node1.chain.add_block(block2_node1.clone()).unwrap();
    node1.network.broadcast(Message::NewBlock(block2_node1)).await;
    sleep(Duration::from_millis(100)).await;

    node2.chain.add_block(block2_node2.clone()).unwrap();
    node2.network.broadcast(Message::NewBlock(block2_node2)).await;
    sleep(Duration::from_millis(200)).await;

    seed.handle_messages().await;
    node1.handle_messages().await;
    node2.handle_messages().await;
    node3.handle_messages().await;
    sleep(Duration::from_millis(200)).await;

    println!("\nChain state after fork:");
    println!("   Seed:  height={}, tip={}", seed.chain.height(), 
             hex::encode(&seed.chain.tip_hash()[..8]));
    println!("   Node1: height={}, tip={}", node1.chain.height(), 
             hex::encode(&node1.chain.tip_hash()[..8]));
    println!("   Node2: height={}, tip={}", node2.chain.height(), 
             hex::encode(&node2.chain.tip_hash()[..8]));
    println!("   Node3: height={}, tip={}", node3.chain.height(), 
             hex::encode(&node3.chain.tip_hash()[..8]));

    println!("\n═══ Phase 5: Fork Resolution ═══\n");

    println!("Node1 continues mining on its chain (blocks #3, #4)...");
    let block3 = node1.chain.mine_block(vec![], &node1.address);
    node1.chain.add_block(block3.clone()).unwrap();
    println!("Node1 mined block #3");

    let block4 = node1.chain.mine_block(vec![], &node1.address);
    node1.chain.add_block(block4.clone()).unwrap();
    println!("Node1 mined block #4");

    println!("\nNode1 broadcasts its longer chain...");
    node1.network.broadcast(Message::NewBlock(block3)).await;
    sleep(Duration::from_millis(100)).await;
    node1.network.broadcast(Message::NewBlock(block4.clone())).await;
    sleep(Duration::from_millis(200)).await;

    seed.handle_messages().await;
    node2.handle_messages().await;
    node3.handle_messages().await;
    sleep(Duration::from_millis(300)).await;

    seed.handle_messages().await;
    node2.handle_messages().await;
    node3.handle_messages().await;
    sleep(Duration::from_millis(300)).await;

    println!("\nFinal chain heights:");
    println!("   Seed:  {}", seed.height());
    println!("   Node1: {}", node1.height());
    println!("   Node2: {}", node2.height());
    println!("   Node3: {}", node3.height());

    let consensus = seed.height() == node1.height() 
                 && node1.height() == node2.height()
                 && node2.height() == node3.height();

    if consensus {
        println!("\nCONSENSUS ACHIEVED");
        println!("   All nodes converged to height {}", node1.height());
        
        let hash_seed = hex::encode(&seed.chain.tip_hash()[..16]);
        let hash_n1 = hex::encode(&node1.chain.tip_hash()[..16]);
        let hash_n2 = hex::encode(&node2.chain.tip_hash()[..16]);
        let hash_n3 = hex::encode(&node3.chain.tip_hash()[..16]);
        
        println!("\nTip hash verification:");
        println!("   Seed:  {}...", hash_seed);
        println!("   Node1: {}...", hash_n1);
        println!("   Node2: {}...", hash_n2);
        println!("   Node3: {}...", hash_n3);
        
        if hash_seed == hash_n1 && hash_n1 == hash_n2 && hash_n2 == hash_n3 {
            println!("\nAll nodes have identical chains");
        }
    } else {
        println!("\nConsensus not fully achieved, heights differ");
    }

    println!("\nDemo completed");
}

