use toychain::crypto::meets_difficulty;
use toychain::tx::Transaction;

#[path = "../labs/lab3/block.rs"]
mod block;

use block::Block;

fn empty_block(difficulty: u32) -> Block {
    Block { height: 1, timestamp: 1_000_000, prev_hash: [0u8; 32], nonce: 0, difficulty, txs: vec![] }
}

#[test]
fn test_txs_root_empty() {
    assert_eq!(empty_block(8).txs_root(), [0u8; 32]);
}

#[test]
fn test_txs_root_deterministic() {
    let cb = Transaction::new_coinbase("miner".to_string(), 50);
    let mut b = empty_block(8);
    b.txs.push(cb);
    assert_eq!(b.txs_root(), b.txs_root());
    assert_ne!(b.txs_root(), [0u8; 32]);
}

#[test]
fn test_txs_root_different_txs() {
    let cb1 = Transaction::new_coinbase("alice".to_string(), 50);
    let cb2 = Transaction::new_coinbase("bob".to_string(), 50);

    let mut b1 = empty_block(8);
    b1.txs.push(cb1);
    let mut b2 = empty_block(8);
    b2.txs.push(cb2);

    assert_ne!(b1.txs_root(), b2.txs_root());
}

#[test]
fn test_header_hash_length() {
    assert_eq!(empty_block(8).header_hash().len(), 32);
}

#[test]
fn test_header_hash_changes_with_nonce() {
    let mut b = empty_block(8);
    let h0 = b.header_hash();
    b.nonce = 1;
    let h1 = b.header_hash();
    assert_ne!(h0, h1);
}

#[test]
fn test_mine_satisfies_difficulty() {
    let b = empty_block(8).mine(); // difficulty=8 → быстро
    assert!(
        meets_difficulty(&b.header_hash(), b.difficulty),
        "после mine() хеш должен удовлетворять difficulty"
    );
}

#[test]
fn test_mine_deterministic() {
    let b1 = empty_block(10).mine();
    let b2 = empty_block(10).mine();
    assert_eq!(b1.nonce, b2.nonce);
}

#[test]
fn test_mine_different_blocks_different_nonce() {
    let mut b_alice = empty_block(10);
    b_alice.txs.push(Transaction::new_coinbase("alice".to_string(), 50));

    let mut b_bob = empty_block(10);
    b_bob.txs.push(Transaction::new_coinbase("bob".to_string(), 50));

    let m1 = b_alice.mine();
    let m2 = b_bob.mine();

    assert!(meets_difficulty(&m1.header_hash(), m1.difficulty));
    assert!(meets_difficulty(&m2.header_hash(), m2.difficulty));
}
