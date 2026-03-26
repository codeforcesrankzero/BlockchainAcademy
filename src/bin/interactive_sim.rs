use std::fs;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use toychain::chain::Chain;
use toychain::mempool::Mempool;
use toychain::crypto::pubkey_to_address;
use toychain::tx::Transaction;
use ed25519_dalek::Keypair;
use rand::rngs::OsRng;
use rand::{thread_rng, Rng};

#[derive(Debug, Deserialize)]
struct Config {
    blockchain: BlockchainConfig,
    simulation: SimulationConfig,
    wallets: WalletsConfig,
    scenarios: ScenariosConfig,
    visualization: VisualizationConfig,
}

#[derive(Debug, Deserialize)]
struct BlockchainConfig {
    difficulty: u32,
    block_reward: u64,
    max_txs_per_block: usize,
}

#[derive(Debug, Deserialize)]
struct SimulationConfig {
    duration_rounds: usize,
    tx_creation_probability: f64,
    mining_rounds_per_node: Vec<usize>,
    invalid_tx_probability: f64,
}

#[derive(Debug, Deserialize)]
struct WalletsConfig {
    initial_count: usize,
    names: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ScenariosConfig {
    active: String,
}

#[derive(Debug, Deserialize)]
struct VisualizationConfig {
    show_mempool: bool,
    show_balances: bool,
    show_network_topology: bool,
    show_block_details: bool,
    log_level: String,
}

struct Wallet {
    name: String,
    kp: Keypair,
    addr: String,
}

impl Wallet {
    fn new(name: String) -> Self {
        let mut rng = OsRng;
        let kp = Keypair::generate(&mut rng);
        let addr = pubkey_to_address(&kp.public);
        Self { name, kp, addr }
    }

    fn make_tx(&self, to_addr: &str, amount: u64, nonce: u64) -> Transaction {
        let mut tx = Transaction::new_unsigned(
            self.addr.clone(),
            to_addr.to_string(),
            amount,
            nonce,
            self.kp.public.as_bytes().to_vec(),
        );
        tx.sign(&self.kp);
        tx
    }
}

struct Miner {
    name: String,
    wallet: Wallet,
    hash_power: usize,
}

impl Miner {
    fn new(name: String, hash_power: usize) -> Self {
        let wallet = Wallet::new(name.clone());
        Self { name, wallet, hash_power }
    }
}

struct SimulationStats {
    total_blocks: u64,
    total_txs: usize,
    failed_txs: usize,
    miner_blocks: HashMap<String, u64>,
}

impl SimulationStats {
    fn new() -> Self {
        Self {
            total_blocks: 0,
            total_txs: 0,
            failed_txs: 0,
            miner_blocks: HashMap::new(),
        }
    }

    fn display(&self) {
        println!("\n╔════════════════════════════════════════╗");
        println!("║       SIMULATION STATISTICS            ║");
        println!("╚════════════════════════════════════════╝");
        println!("Total blocks mined: {}", self.total_blocks);
        println!("Total transactions: {}", self.total_txs);
        println!("Failed transactions: {}", self.failed_txs);
        println!("\nBlocks by miner:");
        for (miner, count) in &self.miner_blocks {
            let percentage = (*count as f64 / self.total_blocks as f64) * 100.0;
            println!("  {}: {} ({:.1}%)", miner, count, percentage);
        }
    }
}

fn visualize_chain_state(
    chain: &Chain,
    mempool: &Mempool,
    users: &[Miner],
    config: &VisualizationConfig,
) {
    println!("\n┌─────────────────────────────────────────┐");
    println!("│         BLOCKCHAIN STATE                │");
    println!("└─────────────────────────────────────────┘");
    
    println!("Height: {}", chain.height());
    println!("Difficulty: {}", chain.difficulty);
    
    if config.show_block_details {
        let last_block = chain.blocks.last().unwrap();
        println!("\nLast block:");
        println!("  Hash: {}...", hex::encode(&last_block.header_hash()[..16]));
        println!("  Transactions: {}", last_block.txs.len());
        println!("  Timestamp: {}", last_block.timestamp);
    }

    if config.show_mempool {
        println!("\nMempool: {} pending transactions", mempool.len());
    }

    if config.show_balances {
        println!("\nUser Balances:");
        let mut total: u128 = 0;
        for user in users {
            let balance = chain.state.balance_of(&user.wallet.addr);
            total += balance;
            println!("  {}: {} (hash power: {})", user.name, balance, user.hash_power);
        }
        
        let expected = chain.height() * chain.block_reward;
        println!("\n  Total: {} (expected: {})", total, expected);
        if total == expected as u128 {
            println!("  Money supply is correct");
        } else {
            println!("  MONEY SUPPLY MISMATCH (diff: {})", (expected as i128) - (total as i128));
        }
    }
}

fn visualize_round_header(round: usize, max_rounds: usize) {
    println!("\n═══════════════════════════════════════════════════");
    println!("         ROUND {}/{}", round, max_rounds);
    println!("═══════════════════════════════════════════════════");
}

fn load_config() -> Config {
    let config_str = fs::read_to_string("simulation_config.toml")
        .expect("Failed to read simulation_config.toml");
    toml::from_str(&config_str).expect("Failed to parse config")
}

fn main() {
    println!("╔════════════════════════════════════════════════╗");
    println!("║   ToyChain Interactive Simulation             ║");
    println!("║   Configurable Blockchain Visualization       ║");
    println!("╚════════════════════════════════════════════════╝\n");

    let config = load_config();
    
    println!("Configuration loaded:");
    println!("  Scenario: {}", config.scenarios.active);
    println!("  Rounds: {}", config.simulation.duration_rounds);
    println!("  Wallets: {}", config.wallets.initial_count);
    println!("  Difficulty: {}", config.blockchain.difficulty);
    println!("  Block reward: {}", config.blockchain.block_reward);

    let mut chain = Chain::new(
        config.blockchain.difficulty,
        config.blockchain.block_reward,
    );
    chain.genesis();

    let mut mempool = Mempool::new();
    let mut stats = SimulationStats::new();

    let users: Vec<Miner> = config.wallets.names
        .iter()
        .zip(&config.simulation.mining_rounds_per_node)
        .map(|(name, &power)| Miner::new(name.clone(), power))
        .collect();

    println!("\nNetwork Participants (miners & users):");
    for user in &users {
        println!("  {}: hash power = {}", user.name, user.hash_power);
    }

    let mut rng = thread_rng();

    println!("\nStarting simulation...\n");
    std::thread::sleep(std::time::Duration::from_secs(1));

    for round in 1..=config.simulation.duration_rounds {
        visualize_round_header(round, config.simulation.duration_rounds);

        if rng.gen_bool(config.simulation.tx_creation_probability) {
            let sender_idx = rng.gen_range(0, users.len());
            let mut receiver_idx = rng.gen_range(0, users.len());
            while receiver_idx == sender_idx {
                receiver_idx = rng.gen_range(0, users.len());
            }

            let sender = &users[sender_idx];
            let receiver = &users[receiver_idx];

            let balance = chain.state.balance_of(&sender.wallet.addr);
            if balance > 0 {
                let max_amount = std::cmp::min(50u128, balance) as u64;
                let amount = rng.gen_range(1, max_amount + 1);
                let nonce = chain.state.nonce_of(&sender.wallet.addr);

                let mut tx = sender.wallet.make_tx(&receiver.wallet.addr, amount, nonce);

                if rng.gen_bool(config.simulation.invalid_tx_probability) {
                    tx.signature[0] ^= 0x01;
                    println!("{} created INVALID tx → {} ({})", 
                             sender.name, receiver.name, amount);
                } else {
                    println!("{} created tx → {} ({})", 
                             sender.name, receiver.name, amount);
                }

                match mempool.add_tx(tx, &chain.state) {
                    Ok(_) => {
                        println!("   Added to mempool (size: {})", mempool.len());
                        stats.total_txs += 1;
                    }
                    Err(e) => {
                        println!("   Rejected: {}", e);
                        stats.failed_txs += 1;
                    }
                }
            }
        }

        let total_hash_power: usize = users.iter().map(|m| m.hash_power).sum();
        let random_power = rng.gen_range(0, total_hash_power);
        
        let mut cumulative_power = 0;
        let mut selected_miner = &users[0];
        
        for user in &users {
            cumulative_power += user.hash_power;
            if random_power < cumulative_power {
                selected_miner = user;
                break;
            }
        }

        println!("\n{} is mining...", selected_miner.name);

        let txs_to_mine = mempool.select_txs(&chain.state, config.blockchain.max_txs_per_block);
        println!("   Selected {} transactions from mempool", txs_to_mine.len());

        let block = chain.mine_block(txs_to_mine.clone(), &selected_miner.wallet.addr);
        
        match chain.add_block(block.clone()) {
            Ok(_) => {
                println!("   Block #{} mined", chain.height());
                
                for tx in &block.txs {
                    if !tx.is_coinbase() {
                        mempool.remove_by_nonce(&tx.from, tx.nonce);
                    }
                }

                stats.total_blocks += 1;
                *stats.miner_blocks.entry(selected_miner.name.clone()).or_insert(0) += 1;
            }
            Err(e) => {
                println!("   Block rejected: {}", e);
            }
        }

        visualize_chain_state(&chain, &mempool, &users, &config.visualization);

        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    stats.display();

    println!("\n╔════════════════════════════════════════╗");
    println!("║     SIMULATION COMPLETED               ║");
    println!("╚════════════════════════════════════════╝");
}

