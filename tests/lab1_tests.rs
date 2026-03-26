use ed25519_dalek::Keypair;
use rand::rngs::OsRng;

#[path = "../labs/lab1/crypto.rs"]
mod crypto;

use crypto::{sha256, meets_difficulty, pubkey_to_address};

#[test]
fn test_sha256_deterministic() {
    let h1 = sha256(b"hello");
    let h2 = sha256(b"hello");
    assert_eq!(h1, h2);
    assert_eq!(h1.len(), 32);
}

#[test]
fn test_sha256_avalanche() {
    let h1 = sha256(b"hello");
    let h2 = sha256(b"Hello");
    assert_ne!(h1, h2);
    let diff = h1.iter().zip(h2.iter()).filter(|(a, b)| a != b).count();
    assert!(diff > 10, "слишком мало отличий — лавинного эффекта нет");
}

#[test]
fn test_sha256_matches_reference() {
    for input in [b"abc".as_ref(), b"hello world", b"blockchain academy", b""] {
        let ours = sha256(input);
        let reference = toychain::crypto::sha256(input);
        assert_eq!(ours, reference, "sha256({:?}) должен совпадать с reference", input);
    }
}

#[test]
fn test_meets_difficulty_zero() {
    let hash = [0xffu8; 32];
    assert!(meets_difficulty(&hash, 0));
}

#[test]
fn test_meets_difficulty_8() {
    let mut hash = [0u8; 32];
    hash[0] = 0x00;
    assert!(meets_difficulty(&hash, 8));

    hash[0] = 0x01;
    assert!(!meets_difficulty(&hash, 8));
}

#[test]
fn test_meets_difficulty_12() {
    let mut hash = [0u8; 32];
    hash[1] = 0x0f;
    assert!(meets_difficulty(&hash, 12));

    hash[1] = 0x10;
    assert!(!meets_difficulty(&hash, 12));
}

#[test]
fn test_pubkey_to_address_format() {
    let mut rng = OsRng;
    let kp = Keypair::generate(&mut rng);
    let addr = pubkey_to_address(&kp.public);

    assert_eq!(addr.len(), 64, "адрес должен быть 64 hex-символа (32 байта)");
    assert!(addr.chars().all(|c| c.is_ascii_hexdigit()), "только hex символы");
}

#[test]
fn test_pubkey_to_address_unique() {
    let mut rng = OsRng;
    let kp1 = Keypair::generate(&mut rng);
    let kp2 = Keypair::generate(&mut rng);
    assert_ne!(pubkey_to_address(&kp1.public), pubkey_to_address(&kp2.public));
}
