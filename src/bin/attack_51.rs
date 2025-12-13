use toychain::chain::Chain;
use toychain::crypto::pubkey_to_address;
use toychain::tx::Transaction;
use ed25519_dalek::Keypair;
use rand::rngs::OsRng;

struct Attacker {
    name: &'static str,
    keypair: Keypair,
    address: String,
}

impl Attacker {
    fn new(name: &'static str) -> Self {
        let keypair = Keypair::generate(&mut OsRng);
        let address = pubkey_to_address(&keypair.public);
        Self { name, keypair, address }
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
    println!("51% Attack Simulation\n");

    let attacker = Attacker::new("Attacker");
    let merchant = Attacker::new("Merchant");
    let honest_miner = "honest_miner";
    
    println!("Attacker: {}", &attacker.address[..16]);
    println!("Merchant: {}", &merchant.address[..16]);

    println!("\nPhase 1: Honest Network");
    
    let mut honest_chain = Chain::new(8, 50);
    honest_chain.genesis();

    for _ in 1..=20 {
        let block = honest_chain.mine_block(vec![], &attacker.address);
        honest_chain.add_block(block).unwrap();
    }
    
    println!("Height: {}, Attacker balance: {}", 
        honest_chain.height(),
        honest_chain.state.balance_of(&attacker.address));

    println!("\nPhase 2: Purchase");
    
    let payment_tx = attacker.make_tx(&merchant.address, 900, 0);
    let payment_block = honest_chain.mine_block(vec![payment_tx.clone()], honest_miner);
    honest_chain.add_block(payment_block).unwrap();
    
    println!("Payment block mined at height {}", honest_chain.height());
    println!("Attacker: {}, Merchant: {}", 
        honest_chain.state.balance_of(&attacker.address),
        honest_chain.state.balance_of(&merchant.address));

    println!("\nPhase 3: Attack (mining alternate chain)");
    
    let mut attacker_chain = Chain::new(8, 50);
    attacker_chain.blocks = honest_chain.blocks[..honest_chain.blocks.len()-1].to_vec();
    
    for block in &attacker_chain.blocks {
        attacker_chain.state.apply_block(block).unwrap();
    }
    
    let fraud_tx = attacker.make_tx(&attacker.address, 900, 0);
    let fraud_block = attacker_chain.mine_block(vec![fraud_tx], &attacker.address);
    attacker_chain.add_block(fraud_block).unwrap();

    let block = honest_chain.mine_block(vec![], honest_miner);
    honest_chain.add_block(block).unwrap();

    for _ in 0..2 {
        let block = attacker_chain.mine_block(vec![], &attacker.address);
        attacker_chain.add_block(block).unwrap();
    }

    println!("\nPhase 4: Chain Comparison");
    
    println!("Honest chain height: {}", honest_chain.height());
    println!("Attack chain height: {}", attacker_chain.height());
    
    if attacker_chain.height() > honest_chain.height() {
        println!("\nAttack successful (longer chain wins)");
        println!("Attacker balance: {} (recovered)", 
            attacker_chain.state.balance_of(&attacker.address));
        println!("Merchant balance: {} (payment vanished)", 
            attacker_chain.state.balance_of(&merchant.address));
    } else {
        println!("\nAttack failed (honest chain still longer)");
    }

    println!("\nProtection:");
    println!("- Wait for multiple confirmations (6+ blocks)");
    println!("- Increase decentralization");
    println!("- Use Proof-of-Stake");
}
