use toychain::chain::Chain;
use toychain::crypto::pubkey_to_address;
use toychain::state::State;
use toychain::tx::{Transaction, TxError};

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

struct Mempool {
    pool: Vec<Transaction>,
}

impl Mempool {
    fn new() -> Self {
        Self { pool: Vec::new() }
    }

    fn len(&self) -> usize {
        self.pool.len()
    }

    fn add_tx(&mut self, tx: Transaction, state: &State) -> Result<(), TxError> {
        tx.verify()?;
        if tx.nonce != state.nonce_of(&tx.from) {
            return Err(TxError::BadNonce);
        }
        if state.balance_of(&tx.from) < tx.amount as u128 {
            return Err(TxError::InsufficientFunds);
        }
        self.pool.push(tx);
        Ok(())
    }

    fn select_for_block(
        &mut self,
        chain: &Chain,
        max: usize,
    ) -> (Vec<Transaction>, usize, usize) {
        let mut included = Vec::new();
        let mut keep = Vec::new();
        let mut tmp = chain.state.clone();
        let mut skipped_nonce = 0usize;
        let mut skipped_funds = 0usize;

        for tx in self.pool.drain(..) {
            if included.len() >= max {
                keep.push(tx);
                continue;
            }
            match tmp.apply_tx(&tx) {
                Ok(_) => {
                    included.push(tx);
                }
                Err(TxError::BadNonce) => {
                    skipped_nonce += 1;
                    keep.push(tx);
                }
                Err(TxError::InsufficientFunds) => {
                    skipped_funds += 1;
                    keep.push(tx);
                }
                Err(_) => {}
            }
        }
        self.pool = keep;
        (included, skipped_nonce, skipped_funds)
    }

    fn remove_included(&mut self, included_hashes: &[toychain::crypto::Hash32]) {
        self.pool.retain(|tx| {
            let h = tx.hash();
            !included_hashes.iter().any(|x| x == &h)
        });
    }
}

fn main() {
    let mut chain = Chain::new(12, 50);
    chain.genesis();
    let mut mempool = Mempool::new();

    let miners = vec![Miner::new("Miner1"), Miner::new("Miner2")];

    let alice = Wallet::new("Alice");
    let bob = Wallet::new("Bob");
    let carol = Wallet::new("Carol");

    println!("=== Initial Setup ===");
    println!("Block reward: {}", chain.block_reward);
    println!("All start with 0 balance\n");

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
        println!("\n=== Round {} ===", round);

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
                    println!(
                        "TX (BROKEN SIG) {} -> {} amount={}",
                        sender.name, recv.name, amount
                    );
                } else if rng.gen_bool(0.15) {
                    let bad_nonce = tx.nonce + 1;
                    tx = sender.make_tx_with_nonce(recv.address(), amount, bad_nonce);
                    println!(
                        "TX (BAD NONCE) {} -> {} amount={}",
                        sender.name, recv.name, amount
                    );
                } else {
                    println!("TX {} -> {} amount={}", sender.name, recv.name, amount);
                }

                match mempool.add_tx(tx, &chain.state) {
                    Ok(_) => println!("  -> accepted (mempool size={})", mempool.len()),
                    Err(e) => println!("  -> rejected: {}", e),
                }
            }
        }

        let miner_idx = rng.gen_range(0, miners.len());
        let miner = &miners[miner_idx];

        let (to_include, skipped_nonce, skipped_funds) =
            mempool.select_for_block(&chain, max_txs_per_block);

        let block = chain.mine_block(to_include, miner.address());
        println!("\nMining block (miner: {})...", miner.name);
        chain.add_block(block).expect("valid block");

        mempool.remove_included(&[]);

        println!("Block added! Height={}", chain.height());
        println!("  Miner1: {}", chain.state.balance_of(miners[0].address()));
        println!("  Miner2: {}", chain.state.balance_of(miners[1].address()));
        println!("  Alice: {}", chain.state.balance_of(alice.address()));
        println!("  Bob: {}", chain.state.balance_of(bob.address()));
        println!("  Carol: {}", chain.state.balance_of(carol.address()));
        println!(
            "Mempool: {}, skipped: nonce={}, funds={}",
            mempool.len(),
            skipped_nonce,
            skipped_funds
        );
    }

    println!("\n=== Final State ===");
    println!("Height: {}", chain.height());
    println!("Total money supply: {}", chain.height() * chain.block_reward);
    println!("\nBalances:");
    println!("  Miner1: {}", chain.state.balance_of(miners[0].address()));
    println!("  Miner2: {}", chain.state.balance_of(miners[1].address()));
    println!("  Alice: {}", chain.state.balance_of(alice.address()));
    println!("  Bob: {}", chain.state.balance_of(bob.address()));
    println!("  Carol: {}", chain.state.balance_of(carol.address()));

    let total_balance = chain.state.balance_of(miners[0].address())
        + chain.state.balance_of(miners[1].address())
        + chain.state.balance_of(alice.address())
        + chain.state.balance_of(bob.address())
        + chain.state.balance_of(carol.address());
    println!("\nTotal balances: {} (should equal {})", total_balance, chain.height() * chain.block_reward);
}
