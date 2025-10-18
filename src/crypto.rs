use sha2::{Digest, Sha256};
use ed25519_dalek::PublicKey;
use std::time::{SystemTime, UNIX_EPOCH};

pub type Hash32 = [u8; 32];

pub fn now_ts() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}

pub fn sha256(data: &[u8]) -> Hash32 {
    let mut h = Sha256::new();
    h.update(data);
    h.finalize().into()
}

pub fn pubkey_to_address(pk: &PublicKey) -> String {
    hex::encode(sha256(pk.as_bytes()))
}

pub fn meets_difficulty(hash: &Hash32, difficulty_bits: u32) -> bool {
    let full_zero_bytes = (difficulty_bits / 8) as usize;
    let rem_bits = (difficulty_bits % 8) as u8;

    if hash.iter().take(full_zero_bytes).any(|&b| b != 0) {
        return false;
    }
    if rem_bits > 0 {
        let next = hash[full_zero_bytes];
        let mask = 0xFFu8 << (8 - rem_bits);
        if next & mask != 0 {
            return false;
        }
    }
    true
}
