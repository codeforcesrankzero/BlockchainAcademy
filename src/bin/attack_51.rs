use toychain::chain::Chain;
use toychain::crypto::pubkey_to_address;
use toychain::tx::Transaction;
use toychain::viz::{self, VizEvent};
use ed25519_dalek::Keypair;
use rand::rngs::OsRng;
use std::thread::sleep;
use std::time::Duration;

struct Wallet {
    keypair:     Keypair,
    pub address: String,
}

impl Wallet {
    fn new() -> Self {
        let keypair = Keypair::generate(&mut OsRng);
        let address = pubkey_to_address(&keypair.public);
        Self { keypair, address }
    }

    fn make_tx(&self, to: &str, amount: u64, nonce: u64) -> Transaction {
        let mut tx = Transaction::new_unsigned(
            self.address.clone(),
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

    let attacker = Wallet::new();
    let merchant = Wallet::new();
    let honest_miner = "honest_miner_addr";

    viz::register(&attacker.address, "Attacker");
    viz::register(&merchant.address, "Merchant");
    viz::register(honest_miner, "HonestMiner");

    let mut honest_chain = Chain::new(8, 50);
    honest_chain.genesis();
    sleep(Duration::from_millis(600));

    for _ in 1..=10 {
        let block = honest_chain.mine_block(vec![], &attacker.address);
        honest_chain.add_block(block).unwrap();
        sleep(Duration::from_millis(400));
    }

    let payment_tx = attacker.make_tx(&merchant.address, 400, 0);
    let payment_block = honest_chain.mine_block(vec![payment_tx], honest_miner);
    let fork_height = payment_block.height;
    honest_chain.add_block(payment_block).unwrap();
    sleep(Duration::from_millis(600));

    for _ in 0..2 {
        let b = honest_chain.mine_block(vec![], honest_miner);
        honest_chain.add_block(b).unwrap();
        sleep(Duration::from_millis(400));
    }

    viz::set_chain("attack");

    let mut attack_chain = Chain::new(8, 50);
    attack_chain.blocks = honest_chain.blocks[..fork_height as usize].to_vec();
    for block in &attack_chain.blocks {
        attack_chain.state.apply_block(block).unwrap();
    }

    let doublespend_tx = attacker.make_tx(&attacker.address, 400, 0);
    let fork_block = attack_chain.mine_block(vec![doublespend_tx], &attacker.address);
    attack_chain.add_block(fork_block).unwrap();
    sleep(Duration::from_millis(500));

    for _ in 0..4 {
        let b = attack_chain.mine_block(vec![], &attacker.address);
        attack_chain.add_block(b).unwrap();
        sleep(Duration::from_millis(400));
    }

    viz::set_chain("main");

    let honest_h  = honest_chain.height();
    let attack_h  = attack_chain.height();
    let won = attack_h > honest_h;

    viz::emit(VizEvent::ForkOutcome {
        attacker_won:  won,
        honest_height: honest_h,
        attack_height: attack_h,
    });

    if won {
        let atk_bal = attack_chain.state.balance_of(&attacker.address);
        let mer_bal = attack_chain.state.balance_of(&merchant.address);
        viz::emit(VizEvent::BalanceChange { addr: attacker.address.clone(), old_bal: 0, new_bal: atk_bal });
        viz::emit(VizEvent::BalanceChange { addr: merchant.address.clone(), old_bal: 400, new_bal: mer_bal });
    }

    viz::flush();
}
