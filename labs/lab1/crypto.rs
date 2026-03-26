use sha2::{Sha256, Digest};
use ed25519_dalek::PublicKey;

pub type Hash32 = [u8; 32];

pub fn sha256(data: &[u8]) -> Hash32 {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

pub fn meets_difficulty(hash: &Hash32, difficulty: u32) -> bool {
    let full_bytes = (difficulty / 8) as usize;
    let remainder  = difficulty % 8;

    for i in 0..full_bytes {
        if hash[i] != 0 { return false; }
    }
    if remainder > 0 && full_bytes < 32 {
        let mask = 0xFF_u8 << (8 - remainder);
        if hash[full_bytes] & mask != 0 { return false; }
    }
    true
}

pub fn pubkey_to_address(pk: &PublicKey) -> String {
    hex::encode(sha256(pk.as_bytes()))
}

pub fn now_ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
