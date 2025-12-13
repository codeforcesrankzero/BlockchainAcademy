use toychain::crypto::{pubkey_to_address, meets_difficulty};
use ed25519_dalek::Keypair;
use rand::rngs::OsRng;

// Подключаем лабораторные модули из labs/lab1/
#[path = "../labs/lab1/transaction.rs"]
mod transaction;

#[path = "../labs/lab1/block.rs"]
mod block;

use transaction::{Transaction, TxError};
use block::Block;

// ============================================
// TESTS FOR TRANSACTION
// ============================================

#[test]
fn test_transaction_creation() {
    let mut rng = OsRng;
    let keypair = Keypair::generate(&mut rng);
    let from_addr = pubkey_to_address(&keypair.public);

    let tx = Transaction::new_unsigned(
        from_addr.clone(),
        "recipient_address".to_string(),
        100,
        0,
        keypair.public.as_bytes().to_vec(),
    );

    assert_eq!(tx.from, from_addr);
    assert_eq!(tx.to, "recipient_address");
    assert_eq!(tx.amount, 100);
    assert_eq!(tx.nonce, 0);
    assert_eq!(tx.signature.len(), 0); // Не подписана
}

#[test]
fn test_transaction_signing() {
    let mut rng = OsRng;
    let keypair = Keypair::generate(&mut rng);
    let from_addr = pubkey_to_address(&keypair.public);

    let mut tx = Transaction::new_unsigned(
        from_addr,
        "recipient_address".to_string(),
        100,
        0,
        keypair.public.as_bytes().to_vec(),
    );

    tx.sign(&keypair);

    assert_eq!(tx.signature.len(), 64); // Ed25519 signature
}

#[test]
fn test_transaction_verification() {
    let mut rng = OsRng;
    let keypair = Keypair::generate(&mut rng);
    let from_addr = pubkey_to_address(&keypair.public);

    let mut tx = Transaction::new_unsigned(
        from_addr,
        "recipient_address".to_string(),
        100,
        0,
        keypair.public.as_bytes().to_vec(),
    );

    tx.sign(&keypair);

    assert!(tx.verify().is_ok());
}

#[test]
fn test_transaction_invalid_signature() {
    let mut rng = OsRng;
    let keypair = Keypair::generate(&mut rng);
    let from_addr = pubkey_to_address(&keypair.public);

    let mut tx = Transaction::new_unsigned(
        from_addr,
        "recipient_address".to_string(),
        100,
        0,
        keypair.public.as_bytes().to_vec(),
    );

    tx.sign(&keypair);

    // Испортим подпись
    tx.signature[0] ^= 0x01;

    assert!(matches!(tx.verify(), Err(TxError::InvalidSignature)));
}

#[test]
fn test_transaction_hash() {
    let mut rng = OsRng;
    let keypair = Keypair::generate(&mut rng);
    let from_addr = pubkey_to_address(&keypair.public);

    let mut tx = Transaction::new_unsigned(
        from_addr,
        "recipient_address".to_string(),
        100,
        0,
        keypair.public.as_bytes().to_vec(),
    );

    tx.sign(&keypair);

    let hash1 = tx.hash();
    let hash2 = tx.hash();

    assert_eq!(hash1, hash2); // Детерминированный хеш
    assert_eq!(hash1.len(), 32); // SHA-256
}

// ============================================
// TESTS FOR BLOCK
// ============================================

#[test]
fn test_block_creation() {
    let block = Block {
        height: 1,
        timestamp: 1234567890,
        prev_hash: [0u8; 32],
        nonce: 0,
        difficulty: 4,
        txs: vec![],
    };

    assert_eq!(block.height, 1);
    assert_eq!(block.txs.len(), 0);
}

#[test]
fn test_block_txs_root() {
    let mut rng = OsRng;
    let keypair = Keypair::generate(&mut rng);
    let from_addr = pubkey_to_address(&keypair.public);

    let mut tx = Transaction::new_unsigned(
        from_addr,
        "recipient".to_string(),
        100,
        0,
        keypair.public.as_bytes().to_vec(),
    );
    tx.sign(&keypair);

    let block = Block {
        height: 1,
        timestamp: 1234567890,
        prev_hash: [0u8; 32],
        nonce: 0,
        difficulty: 4,
        txs: vec![tx],
    };

    let root = block.txs_root();
    assert_eq!(root.len(), 32);
}

#[test]
fn test_block_header_hash() {
    let block = Block {
        height: 1,
        timestamp: 1234567890,
        prev_hash: [0u8; 32],
        nonce: 0,
        difficulty: 4,
        txs: vec![],
    };

    let hash = block.header_hash();
    assert_eq!(hash.len(), 32);
}

#[test]
fn test_block_mining() {
    let block = Block {
        height: 1,
        timestamp: 1234567890,
        prev_hash: [0u8; 32],
        nonce: 0,
        difficulty: 8, // Низкая сложность для быстрого теста
        txs: vec![],
    };

    let mined = block.mine();

    assert!(meets_difficulty(&mined.header_hash(), mined.difficulty));
    assert!(mined.nonce > 0); // Nonce был изменен
}

#[test]
fn test_block_mining_deterministic() {
    let block1 = Block {
        height: 1,
        timestamp: 1234567890,
        prev_hash: [0u8; 32],
        nonce: 0,
        difficulty: 8,
        txs: vec![],
    };

    let block2 = block1.clone();

    let mined1 = block1.mine();
    let mined2 = block2.mine();

    // С одинаковыми параметрами должен быть одинаковый результат
    assert_eq!(mined1.nonce, mined2.nonce);
    assert_eq!(mined1.header_hash(), mined2.header_hash());
}

