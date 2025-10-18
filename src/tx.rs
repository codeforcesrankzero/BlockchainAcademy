use serde::{Deserialize, Serialize};
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use thiserror::Error;

use crate::crypto::{sha256, pubkey_to_address, Hash32};

#[derive(Debug, Error)]
pub enum TxError {
    #[error("signature missing")]
    NoSignature,
    #[error("pubkey missing")]
    NoPubKey,
    #[error("invalid signature")]
    InvalidSignature,
    #[error("from address mismatch with pubkey")]
    FromAddressMismatch,
    #[error("insufficient funds")]
    InsufficientFunds,
    #[error("bad nonce")]
    BadNonce,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub nonce: u64,
    pub pubkey: Vec<u8>,
    pub signature: Vec<u8>,
}

impl Transaction {
    pub fn new_unsigned(from: String, to: String, amount: u64, nonce: u64, pubkey: Vec<u8>) -> Self {
        Self { from, to, amount, nonce, pubkey, signature: Vec::new() }
    }

    pub fn new_coinbase(to: String, amount: u64) -> Self {
        Self {
            from: String::new(),
            to,
            amount,
            nonce: 0,
            pubkey: Vec::new(),
            signature: Vec::new(),
        }
    }

    pub fn is_coinbase(&self) -> bool {
        self.from.is_empty()
    }

    fn bytes_for_signing(&self) -> Vec<u8> {
        #[derive(Serialize)]
        struct SignView<'a> {
            from: &'a String,
            to: &'a String,
            amount: u64,
            nonce: u64,
            pubkey: &'a Vec<u8>,
        }
        let view = SignView {
            from: &self.from,
            to: &self.to,
            amount: self.amount,
            nonce: self.nonce,
            pubkey: &self.pubkey,
        };
        bincode::serialize(&view).unwrap()
    }

    pub fn sign(&mut self, kp: &Keypair) {
        let msg = self.bytes_for_signing();
        let sig = kp.sign(&msg);
        self.signature = sig.to_bytes().to_vec();
    }

    pub fn verify(&self) -> Result<(), TxError> {
        if self.is_coinbase() {
            return Ok(());
        }

        if self.signature.is_empty() { return Err(TxError::NoSignature); }
        if self.pubkey.is_empty() { return Err(TxError::NoPubKey); }

        let pk = PublicKey::from_bytes(&self.pubkey)
            .map_err(|_| TxError::InvalidSignature)?;
        let derived_addr = pubkey_to_address(&pk);
        if derived_addr != self.from {
            return Err(TxError::FromAddressMismatch);
        }

        let sig = Signature::from_bytes(&self.signature)
            .map_err(|_| TxError::InvalidSignature)?;
        let msg = self.bytes_for_signing();
        pk.verify(&msg, &sig).map_err(|_| TxError::InvalidSignature)
    }

    pub fn hash(&self) -> Hash32 {
        let bytes = bincode::serialize(self).unwrap();
        sha256(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;

    #[test]
    fn sign_and_verify_ok_and_fail_on_mutation() {
        let mut rng = OsRng;
        let kp = Keypair::generate(&mut rng);
        let from = crate::crypto::pubkey_to_address(&kp.public);
        let to = from.clone();

        let mut tx = Transaction::new_unsigned(
            from.clone(), to, 10, 0, kp.public.as_bytes().to_vec()
        );
        tx.sign(&kp);
        assert!(tx.verify().is_ok());

        let mut bad = tx.clone();
        bad.amount += 1;
        assert!(bad.verify().is_err());
    }
}
