use toychain::chain::Chain;
use toychain::crypto::pubkey_to_address;
use toychain::state::State;
use toychain::storage::Storage;
use toychain::tx::Transaction;
use toychain::mempool::Mempool;
use toychain::viz;

use std::thread::sleep;
use std::time::Duration;

const ROUND_DELAY_MS: u64 = 1_500;
const TX_DELAY_MS: u64 = 250;

use ed25519_dalek::Keypair;
use rand::rngs::OsRng;
use rand::{thread_rng, Rng};

struct Wallet {
    name: &'static str,
    kp: Keypair,
    addr: String,
}

impl Wallet {
    fn new(name: &'static str) -> Self {
        let mut rng = OsRng;
        let kp = Keypair::generate(&mut rng);
        let addr = pubkey_to_address(&kp.public);
        Self { name, kp, addr }
    }

    fn address(&self) -> &str {
        &self.addr
    }

    fn make_tx_with_nonce(&self, to_addr: &str, amount: u64, nonce: u64) -> Transaction {
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

    fn make_tx(&self, to_addr: &str, amount: u64, state: &State) -> Transaction {
        let nonce = state.nonce_of(&self.addr);
        self.make_tx_with_nonce(to_addr, amount, nonce)
    }
}

struct Miner {
    name: &'static str,
    wallet: Wallet,
}

impl Miner {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            wallet: Wallet::new(name),
        }
    }

    fn address(&self) -> &str {
        self.wallet.address()
    }
}

fn main() {
    viz::start();

    let storage = Storage::new("blockchain_data");
    
    let mut chain = if storage.chain_exists() {
        match storage.load_chain() {
            Ok(c) => {
                println!("Loaded chain: {} blocks", c.blocks.len());
                c
            }
            Err(e) => {
                println!("Load failed: {}", e);
                let mut c = Chain::new(12, 50);
                c.genesis();
                c
            }
        }
    } else {
        let mut c = Chain::new(12, 50);
        c.genesis();
        c
    };
    
    let mut mempool = Mempool::new();

    let miners = vec![Miner::new("Miner1"), Miner::new("Miner2")];
    let alice = Wallet::new("Alice");
    let bob   = Wallet::new("Bob");
    let carol = Wallet::new("Carol");

    viz::register(miners[0].address(), "Miner1");
    viz::register(miners[1].address(), "Miner2");
    viz::register(alice.address(),     "Alice");
    viz::register(bob.address(),       "Bob");
    viz::register(carol.address(),     "Carol");

    let all_wallets: Vec<&Wallet> = vec![
        &miners[0].wallet,
        &miners[1].wallet,
        &alice,
        &bob,
        &carol,
    ];

    let mut rng = thread_rng();
    let rounds = 20;
    let max_txs_per_block = 5;

    for round in 1..=rounds {
        println!("\nRound {}", round);

        sleep(Duration::from_millis(ROUND_DELAY_MS));

        for _ in 0..4 {
            if rng.gen_bool(0.8) {
                let i = rng.gen_range(0, all_wallets.len());
                let mut j = rng.gen_range(0, all_wallets.len());
                if j == i {
                    j = (j + 1) % all_wallets.len();
                }

                let sender = all_wallets[i];
                let recv = all_wallets[j];

                let bal = chain.state.balance_of(sender.address());
                if bal == 0 {
                    continue;
                }
                let max_amount = std::cmp::min(100u128, bal) as u64;
                let amount = rng.gen_range(1, max_amount + 1);

                let mut tx = sender.make_tx(recv.address(), amount, &chain.state);

                if rng.gen_bool(0.1) && !tx.signature.is_empty() {
                    tx.signature[0] ^= 0x01;
                } else if rng.gen_bool(0.15) {
                    let bad_nonce = tx.nonce + 1;
                    tx = sender.make_tx_with_nonce(recv.address(), amount, bad_nonce);
                }

                match mempool.add_tx(tx, &chain.state) {
                    Ok(_) => {},
                    Err(_) => {},
                }

                sleep(Duration::from_millis(TX_DELAY_MS));
            }
        }

        let miner_idx = rng.gen_range(0, miners.len());
        let miner = &miners[miner_idx];

        let txs = mempool.select_txs(&chain.state, max_txs_per_block);

        let block = chain.mine_block(txs, miner.address());
        chain.add_block(block.clone()).expect("valid block");
        
        if let Err(_) = storage.save_chain(&chain) {
        }

        let included_hashes: Vec<_> = block.txs.iter()
            .filter(|tx| !tx.is_coinbase())
            .map(|tx| tx.hash())
            .collect();
        
        mempool.remove_txs(&included_hashes);

        println!("Height: {}, Miner: {}, Mempool: {}", 
            chain.height(), miner.name, mempool.len());
    }

    viz::flush();

    println!("\nFinal state:");
    println!("Height: {}", chain.height());
    println!("Miner1: {}", chain.state.balance_of(miners[0].address()));
    println!("Miner2: {}", chain.state.balance_of(miners[1].address()));
    println!("Alice: {}", chain.state.balance_of(alice.address()));
    println!("Bob: {}", chain.state.balance_of(bob.address()));
    println!("Carol: {}", chain.state.balance_of(carol.address()));
}
