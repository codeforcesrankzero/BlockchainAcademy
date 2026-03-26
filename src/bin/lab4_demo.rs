use toychain::chain::Chain;
use toychain::crypto::pubkey_to_address;
use toychain::tx::Transaction;
use toychain::viz;
use ed25519_dalek::Keypair;
use rand::rngs::OsRng;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    viz::start();

    let mut rng = OsRng;
    let alice_kp = Keypair::generate(&mut rng);
    let bob_kp   = Keypair::generate(&mut rng);
    let alice = pubkey_to_address(&alice_kp.public);
    let bob   = pubkey_to_address(&bob_kp.public);

    viz::register(&alice, "Alice");
    viz::register(&bob,   "Bob");

    let mut chain = Chain::new(12, 50);
    chain.genesis();

    for round in 1u64..=6 {
        sleep(Duration::from_millis(800));

        let miner = if round % 2 == 0 { &alice } else { &bob };
        let txs = if round >= 3 {
            let sender = if chain.state.balance_of(&alice) >= 20 { &alice_kp } else { &bob_kp };
            let sender_addr = pubkey_to_address(&sender.public);
            let recip = if sender_addr == alice { bob.clone() } else { alice.clone() };
            let nonce = chain.state.nonce_of(&sender_addr);
            let bal = chain.state.balance_of(&sender_addr);
            if bal >= 20 {
                let mut tx = Transaction::new_unsigned(
                    sender_addr, recip, 20, nonce, sender.public.as_bytes().to_vec(),
                );
                tx.sign(sender);
                vec![tx]
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        let block = chain.mine_block(txs, miner);
        chain.add_block(block).expect("valid block");
    }

    viz::flush();
    sleep(Duration::from_secs(2));
    println!("\nAlice: {}", chain.state.balance_of(&alice));
    println!("Bob:   {}", chain.state.balance_of(&bob));
}
