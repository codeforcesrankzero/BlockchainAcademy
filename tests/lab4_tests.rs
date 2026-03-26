use toychain::crypto::pubkey_to_address;
use toychain::tx::Transaction;
use ed25519_dalek::Keypair;
use rand::rngs::OsRng;

#[path = "../labs/lab4/state.rs"]
mod state;
#[path = "../labs/lab4/chain.rs"]
mod chain;

use state::State;
use chain::{Chain, BlockError};

#[test]
fn test_balance_default_zero() {
    assert_eq!(State::default().balance_of("nobody"), 0);
}

#[test]
fn test_nonce_default_zero() {
    assert_eq!(State::default().nonce_of("nobody"), 0);
}

#[test]
fn test_apply_coinbase() {
    let mut s = State::default();
    let cb = Transaction::new_coinbase("miner".to_string(), 50);
    s.apply_tx(&cb).unwrap();
    assert_eq!(s.balance_of("miner"), 50);
}

#[test]
fn test_apply_tx_transfers() {
    let mut rng = OsRng;
    let kp = Keypair::generate(&mut rng);
    let addr = pubkey_to_address(&kp.public);

    let mut s = State::default();
    s.balances.insert(addr.clone(), 1000);

    let mut tx = Transaction::new_unsigned(
        addr.clone(), "bob".to_string(), 300, 0, kp.public.as_bytes().to_vec(),
    );
    tx.sign(&kp);

    s.apply_tx(&tx).unwrap();
    assert_eq!(s.balance_of(&addr), 700);
    assert_eq!(s.balance_of("bob"), 300);
    assert_eq!(s.nonce_of(&addr), 1);
}

#[test]
fn test_apply_tx_increments_nonce() {
    let mut rng = OsRng;
    let kp = Keypair::generate(&mut rng);
    let addr = pubkey_to_address(&kp.public);
    let mut s = State::default();
    s.balances.insert(addr.clone(), 1000);

    for nonce in 0..3 {
        let mut tx = Transaction::new_unsigned(
            addr.clone(), "bob".to_string(), 10, nonce, kp.public.as_bytes().to_vec(),
        );
        tx.sign(&kp);
        s.apply_tx(&tx).unwrap();
    }
    assert_eq!(s.nonce_of(&addr), 3);
}

#[test]
fn test_insufficient_funds() {
    let mut rng = OsRng;
    let kp = Keypair::generate(&mut rng);
    let addr = pubkey_to_address(&kp.public);
    let mut s = State::default();
    s.balances.insert(addr.clone(), 10);

    let mut tx = Transaction::new_unsigned(
        addr, "bob".to_string(), 100, 0, kp.public.as_bytes().to_vec(),
    );
    tx.sign(&kp);
    assert!(s.apply_tx(&tx).is_err());
}

#[test]
fn test_bad_nonce() {
    let mut rng = OsRng;
    let kp = Keypair::generate(&mut rng);
    let addr = pubkey_to_address(&kp.public);
    let mut s = State::default();
    s.balances.insert(addr.clone(), 1000);
    s.nonces.insert(addr.clone(), 5);

    let mut tx = Transaction::new_unsigned(
        addr, "bob".to_string(), 10, 0, kp.public.as_bytes().to_vec(),
    );
    tx.sign(&kp);
    assert!(s.apply_tx(&tx).is_err());
}

#[test]
fn test_genesis_creates_block() {
    let mut c = Chain::new(8, 50);
    c.genesis();
    assert_eq!(c.blocks.len(), 1);
    assert_eq!(c.blocks[0].height, 0);
    assert_eq!(c.height(), 0);
}

#[test]
fn test_mine_and_add_block() {
    let mut c = Chain::new(8, 50);
    c.genesis();
    let b = c.mine_block(vec![], "miner");
    c.add_block(b).unwrap();
    assert_eq!(c.height(), 1);
    assert_eq!(c.state.balance_of("miner"), 50);
}

#[test]
fn test_multiple_blocks_accumulate_reward() {
    let mut c = Chain::new(8, 50);
    c.genesis();
    for _ in 0..4 {
        let b = c.mine_block(vec![], "miner");
        c.add_block(b).unwrap();
    }
    assert_eq!(c.state.balance_of("miner"), 200);
}

#[test]
fn test_reject_bad_height() {
    let mut c = Chain::new(8, 50);
    c.genesis();
    let mut b = c.mine_block(vec![], "miner");
    b.height = 99;
    assert!(matches!(c.add_block(b), Err(BlockError::BadHeight)));
}

#[test]
fn test_reject_bad_prev_hash() {
    let mut c = Chain::new(8, 50);
    c.genesis();
    let mut b = c.mine_block(vec![], "miner");
    b.prev_hash = [0xff; 32];
    let b = b.mine();
    assert!(matches!(c.add_block(b), Err(BlockError::PrevHashMismatch)));
}

#[test]
fn test_reject_bad_pow() {
    let mut c = Chain::new(8, 50);
    c.genesis();
    let mut b = c.mine_block(vec![], "miner");
    b.nonce = 0;
    assert!(matches!(c.add_block(b), Err(BlockError::BadPoW)));
}

#[test]
fn test_tx_in_block_state_update() {
    let mut rng = OsRng;
    let kp = Keypair::generate(&mut rng);
    let alice = pubkey_to_address(&kp.public);

    let mut c = Chain::new(8, 100);
    c.genesis();

    let b = c.mine_block(vec![], &alice);
    c.add_block(b).unwrap();
    assert_eq!(c.state.balance_of(&alice), 100);

    let mut tx = Transaction::new_unsigned(
        alice.clone(), "bob".to_string(), 30, 0, kp.public.as_bytes().to_vec(),
    );
    tx.sign(&kp);

    let b2 = c.mine_block(vec![tx], &alice);
    c.add_block(b2).unwrap();

    assert_eq!(c.state.balance_of(&alice), 170); // 100 - 30 + 100 reward
    assert_eq!(c.state.balance_of("bob"), 30);
}
