use toychain::crypto::pubkey_to_address;
use toychain::tx::Transaction;
use ed25519_dalek::Keypair;
use rand::rngs::OsRng;

#[path = "../labs/lab2/state.rs"]
mod state;

#[path = "../labs/lab2/chain.rs"]
mod chain;

use state::State;
use chain::{Chain, BlockError};

// ============================================
// TESTS FOR STATE
// ============================================

#[test]
fn test_state_balance() {
    let mut state = State::default();
    
    assert_eq!(state.balance_of("alice"), 0);
    
    state.balances.insert("alice".to_string(), 100);
    assert_eq!(state.balance_of("alice"), 100);
}

#[test]
fn test_state_nonce() {
    let mut state = State::default();
    
    assert_eq!(state.nonce_of("alice"), 0);
    
    state.nonces.insert("alice".to_string(), 5);
    assert_eq!(state.nonce_of("alice"), 5);
}

#[test]
fn test_apply_transaction() {
    let mut rng = OsRng;
    let keypair = Keypair::generate(&mut rng);
    let from_addr = pubkey_to_address(&keypair.public);
    
    let mut state = State::default();
    state.balances.insert(from_addr.clone(), 1000);
    
    let mut tx = Transaction::new_unsigned(
        from_addr.clone(),
        "bob".to_string(),
        100,
        0,
        keypair.public.as_bytes().to_vec(),
    );
    tx.sign(&keypair);
    
    assert!(state.apply_tx(&tx).is_ok());
    
    assert_eq!(state.balance_of(&from_addr), 900);
    assert_eq!(state.balance_of("bob"), 100);
    assert_eq!(state.nonce_of(&from_addr), 1);
}

#[test]
fn test_apply_coinbase() {
    let mut state = State::default();
    
    let coinbase = Transaction::new_coinbase("miner".to_string(), 50);
    
    assert!(state.apply_tx(&coinbase).is_ok());
    assert_eq!(state.balance_of("miner"), 50);
}

#[test]
fn test_insufficient_funds() {
    let mut rng = OsRng;
    let keypair = Keypair::generate(&mut rng);
    let from_addr = pubkey_to_address(&keypair.public);
    
    let mut state = State::default();
    state.balances.insert(from_addr.clone(), 50);
    
    let mut tx = Transaction::new_unsigned(
        from_addr,
        "bob".to_string(),
        100,
        0,
        keypair.public.as_bytes().to_vec(),
    );
    tx.sign(&keypair);
    
    assert!(state.apply_tx(&tx).is_err());
}

#[test]
fn test_bad_nonce() {
    let mut rng = OsRng;
    let keypair = Keypair::generate(&mut rng);
    let from_addr = pubkey_to_address(&keypair.public);
    
    let mut state = State::default();
    state.balances.insert(from_addr.clone(), 1000);
    state.nonces.insert(from_addr.clone(), 5);
    
    let mut tx = Transaction::new_unsigned(
        from_addr,
        "bob".to_string(),
        100,
        0, // Неправильный nonce (должен быть 5)
        keypair.public.as_bytes().to_vec(),
    );
    tx.sign(&keypair);
    
    assert!(state.apply_tx(&tx).is_err());
}

// ============================================
// TESTS FOR CHAIN
// ============================================

#[test]
fn test_genesis() {
    let mut chain = Chain::new(8, 50);
    chain.genesis();
    
    assert_eq!(chain.height(), 0);
    assert_eq!(chain.blocks.len(), 1);
    assert_eq!(chain.blocks[0].height, 0);
}

#[test]
fn test_chain_height() {
    let mut chain = Chain::new(8, 50);
    
    assert_eq!(chain.height(), 0);
    
    chain.genesis();
    assert_eq!(chain.height(), 0);
}

#[test]
fn test_add_valid_block() {
    let mut chain = Chain::new(8, 50);
    chain.genesis();
    
    let block = chain.mine_block(vec![], "miner");
    assert!(chain.add_block(block).is_ok());
    
    assert_eq!(chain.height(), 1);
    assert_eq!(chain.state.balance_of("miner"), 50);
}

#[test]
fn test_reject_wrong_height() {
    let mut chain = Chain::new(8, 50);
    chain.genesis();
    
    let mut block = chain.mine_block(vec![], "miner");
    block.height = 5; // Неправильная высота
    
    assert!(matches!(chain.add_block(block), Err(BlockError::BadHeight)));
}

#[test]
fn test_reject_wrong_prev_hash() {
    let mut chain = Chain::new(8, 50);
    chain.genesis();
    
    let mut block = chain.mine_block(vec![], "miner");
    block.prev_hash = [0xff; 32]; // Неправильный prev_hash
    
    // Нужно перемайнить с новым prev_hash
    let mined = block.mine();
    
    assert!(matches!(chain.add_block(mined), Err(BlockError::PrevHashMismatch)));
}

#[test]
fn test_reject_bad_pow() {
    let mut chain = Chain::new(8, 50);
    chain.genesis();
    
    let mut block = chain.mine_block(vec![], "miner");
    block.nonce = 0; // Сбросим nonce - PoW не будет валиден
    
    assert!(matches!(chain.add_block(block), Err(BlockError::BadPoW)));
}

#[test]
fn test_mine_block() {
    let chain = Chain::new(8, 50);
    
    // Создаем fake genesis для теста
    let genesis = toychain::block::Block {
        height: 0,
        timestamp: 0,
        prev_hash: [0; 32],
        nonce: 0,
        difficulty: 8,
        txs: vec![],
    }.mine();
    
    let mut test_chain = chain.clone();
    test_chain.blocks.push(genesis);
    
    let block = test_chain.mine_block(vec![], "miner");
    
    assert_eq!(block.height, 1);
    assert_eq!(block.txs.len(), 1); // Только coinbase
    assert!(block.txs[0].is_coinbase());
    assert_eq!(block.txs[0].amount, 50);
}

#[test]
fn test_multiple_blocks() {
    let mut chain = Chain::new(8, 50);
    chain.genesis();
    
    for i in 1..=5 {
        let block = chain.mine_block(vec![], "miner");
        chain.add_block(block).unwrap();
        assert_eq!(chain.height(), i);
    }
    
    assert_eq!(chain.state.balance_of("miner"), 250); // 5 * 50
}

