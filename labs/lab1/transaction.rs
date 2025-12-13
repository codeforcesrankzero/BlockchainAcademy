use serde::{Deserialize, Serialize};
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use thiserror::Error;

use toychain::crypto::{sha256, pubkey_to_address, Hash32};

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
    pub fn new_unsigned(
        from: String,
        to: String,
        amount: u64,
        nonce: u64,
        pubkey: Vec<u8>,
    ) -> Self {
        todo!("Реализуйте создание транзакции")
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
        struct TxView<'a> {
            from: &'a str,
            to: &'a str,
            amount: u64,
            nonce: u64,
        }
        let view = TxView {
            from: &self.from,
            to: &self.to,
            amount: self.amount,
            nonce: self.nonce,
        };
        bincode::serialize(&view).unwrap()
    }

    pub fn sign(&mut self, keypair: &Keypair) {
        todo!("Реализуйте подписание транзакции")
    }

    pub fn verify(&self) -> Result<(), TxError> {
        if self.is_coinbase() {
            return Ok(());
        }
        
        todo!("Реализуйте проверку подписи")
    }

    pub fn hash(&self) -> Hash32 {
        todo!("Реализуйте хеширование транзакции")
    }
}
