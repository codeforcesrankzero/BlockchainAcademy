use toychain::chain::Chain;
use toychain::crypto::pubkey_to_address;
use toychain::mempool::Mempool;
use toychain::tx::Transaction;
use toychain::viz;

use ed25519_dalek::Keypair;
use rand::rngs::OsRng;

struct Wallet {
    name: &'static str,
    keypair: Keypair,
    pub addr: String,
}

impl Wallet {
    fn new(name: &'static str) -> Self {
        let keypair = Keypair::generate(&mut OsRng);
        let addr = pubkey_to_address(&keypair.public);
        Self { name, keypair, addr }
    }

    fn send(&self, to: &str, amount: u64, nonce: u64) -> Transaction {
        let mut tx = Transaction::new_unsigned(
            self.addr.clone(),
            to.to_string(),
            amount,
            nonce,
            self.keypair.public.as_bytes().to_vec(),
        );
        tx.sign(&self.keypair);
        tx
    }
}

fn main() {
    viz::start();

    println!("\x1b[1m\x1b[36m━━━  TOYCHAIN VISUALIZER DEMO  ━━━\x1b[0m\n");

    let mut chain = Chain::new(8, 50); // difficulty=8 быстро майнится
    let mut mempool = Mempool::new();

    let alice = Wallet::new("Alice");
    let bob   = Wallet::new("Bob");

    separator("STEP 1: genesis");
    chain.genesis();

    separator("STEP 2: Alice mines block #1");
    let b1 = chain.mine_block(vec![], &alice.addr);
    chain.add_block(b1).unwrap();

    separator("STEP 3: submit transactions");

    let tx_good = alice.send(&bob.addr, 20, 0);
    mempool.add_tx(tx_good, &chain.state).unwrap();

    let tx_bad_nonce = alice.send(&bob.addr, 10, 5);
    let _ = mempool.add_tx(tx_bad_nonce, &chain.state);

    let tx_bad_funds = alice.send(&bob.addr, 9999, 0);
    let _ = mempool.add_tx(tx_bad_funds, &chain.state);

    let mut tx_bad_sig = alice.send(&bob.addr, 5, 0);
    if !tx_bad_sig.signature.is_empty() {
        tx_bad_sig.signature[0] ^= 0xff;
    }
    let _ = mempool.add_tx(tx_bad_sig, &chain.state);

    separator("STEP 4: Alice mines block #2 with pending txs");
    let txs = mempool.select_txs(&chain.state, 10);
    let b2 = chain.mine_block(txs, &alice.addr);
    chain.add_block(b2).unwrap();

    let confirmed: Vec<_> = chain.blocks.last().unwrap().txs.iter()
        .filter(|t| !t.is_coinbase())
        .map(|t| t.hash())
        .collect();
    mempool.remove_txs(&confirmed);

    separator("STEP 5: Bob mines block #3");
    let b3 = chain.mine_block(vec![], &bob.addr);
    chain.add_block(b3).unwrap();

    separator("STEP 6: try to add an invalid block (wrong height)");
    use toychain::block::Block;
    use toychain::crypto::now_ts;
    let bad_block = Block {
        height: 1,
        timestamp: now_ts(),
        prev_hash: [0u8; 32],
        nonce: 0,
        difficulty: 8,
        txs: vec![],
    };
    let _ = chain.add_block(bad_block);

    viz::flush();

    println!("\n\x1b[1m\x1b[36m━━━  FINAL STATE  ━━━\x1b[0m");
    println!("  Chain height : {}", chain.height());
    println!("  {}: {} coins", alice.name, chain.state.balance_of(&alice.addr));
    println!("  {}:   {} coins", bob.name,   chain.state.balance_of(&bob.addr));
}

fn separator(label: &str) {
    println!("\n\x1b[2m── {} {}\x1b[0m",
        label,
        "─".repeat(50usize.saturating_sub(label.len() + 4)));
}
