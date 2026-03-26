use ed25519_dalek::Keypair;
use rand::rngs::OsRng;
use toychain::crypto::pubkey_to_address;

#[path = "../labs/lab2/tx.rs"]
mod tx;

use tx::{Transaction, TxError};

fn make_signed(from_kp: &Keypair, to: &str, amount: u64, nonce: u64) -> Transaction {
    let from = pubkey_to_address(&from_kp.public);
    let mut tx = Transaction::new_unsigned(
        from, to.to_string(), amount, nonce, from_kp.public.as_bytes().to_vec(),
    );
    tx.sign(from_kp);
    tx
}

#[test]
fn test_new_unsigned_fields() {
    let mut rng = OsRng;
    let kp = Keypair::generate(&mut rng);
    let addr = pubkey_to_address(&kp.public);

    let tx = Transaction::new_unsigned(
        addr.clone(), "bob".to_string(), 42, 3, kp.public.as_bytes().to_vec(),
    );
    assert_eq!(tx.from, addr);
    assert_eq!(tx.to, "bob");
    assert_eq!(tx.amount, 42);
    assert_eq!(tx.nonce, 3);
    assert!(tx.signature.is_empty(), "новая транзакция не должна быть подписана");
}

#[test]
fn test_sign_produces_64_bytes() {
    let mut rng = OsRng;
    let kp = Keypair::generate(&mut rng);
    let tx = make_signed(&kp, "bob", 10, 0);
    assert_eq!(tx.signature.len(), 64, "Ed25519 подпись = 64 байта");
}

#[test]
fn test_verify_valid() {
    let mut rng = OsRng;
    let kp = Keypair::generate(&mut rng);
    let tx = make_signed(&kp, "bob", 100, 0);
    assert!(tx.verify().is_ok());
}

#[test]
fn test_verify_tampered_amount() {
    let mut rng = OsRng;
    let kp = Keypair::generate(&mut rng);
    let mut tx = make_signed(&kp, "bob", 100, 0);
    tx.amount = 9999; // подделка суммы
    assert!(tx.verify().is_err(), "изменённая сумма должна не проходить верификацию");
}

#[test]
fn test_verify_tampered_signature() {
    let mut rng = OsRng;
    let kp = Keypair::generate(&mut rng);
    let mut tx = make_signed(&kp, "bob", 100, 0);
    tx.signature[0] ^= 0x01;
    assert!(matches!(tx.verify(), Err(TxError::InvalidSignature)));
}

#[test]
fn test_verify_wrong_pubkey() {
    let mut rng = OsRng;
    let kp1 = Keypair::generate(&mut rng);
    let kp2 = Keypair::generate(&mut rng);
    let mut tx = make_signed(&kp1, "bob", 100, 0);
    tx.pubkey = kp2.public.as_bytes().to_vec();
    assert!(tx.verify().is_err());
}

#[test]
fn test_coinbase_always_valid() {
    let cb = Transaction::new_coinbase("miner".to_string(), 50);
    assert!(cb.is_coinbase());
    assert!(cb.verify().is_ok(), "coinbase верифицируется без подписи");
}

#[test]
fn test_hash_deterministic() {
    let mut rng = OsRng;
    let kp = Keypair::generate(&mut rng);
    let tx = make_signed(&kp, "bob", 50, 0);
    assert_eq!(tx.hash(), tx.hash());
    assert_eq!(tx.hash().len(), 32);
}

#[test]
fn test_hash_changes_on_tamper() {
    let mut rng = OsRng;
    let kp = Keypair::generate(&mut rng);
    let tx = make_signed(&kp, "bob", 50, 0);
    let mut tx2 = tx.clone();
    tx2.amount = 51;
    assert_ne!(tx.hash(), tx2.hash());
}
