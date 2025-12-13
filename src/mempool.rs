use std::collections::HashMap;
use crate::tx::{Transaction, TxError};
use crate::state::State;
use crate::crypto::Hash32;

pub struct Mempool {
    txs: HashMap<Hash32, Transaction>,
}

impl Mempool {
    pub fn new() -> Self {
        Self {
            txs: HashMap::new(),
        }
    }

    pub fn add_tx(&mut self, tx: Transaction, state: &State) -> Result<(), TxError> {
        if tx.is_coinbase() {
            return Ok(());
        }

        tx.verify()?;

        if tx.nonce != state.nonce_of(&tx.from) {
            return Err(TxError::BadNonce);
        }

        if state.balance_of(&tx.from) < tx.amount as u128 {
            return Err(TxError::InsufficientFunds);
        }

        let hash = tx.hash();
        self.txs.insert(hash, tx);
        Ok(())
    }

    pub fn has_tx(&self, hash: &Hash32) -> bool {
        self.txs.contains_key(hash)
    }

    pub fn get_tx(&self, hash: &Hash32) -> Option<&Transaction> {
        self.txs.get(hash)
    }

    pub fn len(&self) -> usize {
        self.txs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.txs.is_empty()
    }

    pub fn select_txs(&self, state: &State, max: usize) -> Vec<Transaction> {
        let mut included = Vec::new();
        let mut tmp_state = state.clone();

        for tx in self.txs.values() {
            if included.len() >= max {
                break;
            }

            if tmp_state.apply_tx(tx).is_ok() {
                included.push(tx.clone());
            }
        }

        included
    }

    pub fn remove_txs(&mut self, hashes: &[Hash32]) {
        for hash in hashes {
            self.txs.remove(hash);
        }
    }

    pub fn remove_by_nonce(&mut self, from: &str, nonce: u64) {
        self.txs.retain(|_, tx| {
            !(tx.from == from && tx.nonce <= nonce)
        });
    }

    pub fn get_all_txs(&self) -> Vec<Transaction> {
        self.txs.values().cloned().collect()
    }

    pub fn get_tx_hashes(&self) -> Vec<Hash32> {
        self.txs.keys().cloned().collect()
    }
}

impl Default for Mempool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::pubkey_to_address;
    use ed25519_dalek::Keypair;
    use rand::rngs::OsRng;

    #[test]
    fn test_mempool_basic() {
        let mut mempool = Mempool::new();
        let mut state = State::default();

        let mut rng = OsRng;
        let kp = Keypair::generate(&mut rng);
        let addr = pubkey_to_address(&kp.public);

        state.balances.insert(addr.clone(), 1000);

        let mut tx = Transaction::new_unsigned(
            addr.clone(),
            "recipient".to_string(),
            100,
            0,
            kp.public.as_bytes().to_vec(),
        );
        tx.sign(&kp);

        assert!(mempool.add_tx(tx.clone(), &state).is_ok());
        assert_eq!(mempool.len(), 1);
        assert!(mempool.has_tx(&tx.hash()));
    }

    #[test]
    fn test_mempool_rejects_bad_nonce() {
        let mut mempool = Mempool::new();
        let mut state = State::default();

        let mut rng = OsRng;
        let kp = Keypair::generate(&mut rng);
        let addr = pubkey_to_address(&kp.public);

        state.balances.insert(addr.clone(), 1000);

        let mut tx = Transaction::new_unsigned(
            addr.clone(),
            "recipient".to_string(),
            100,
            5,
            kp.public.as_bytes().to_vec(),
        );
        tx.sign(&kp);

        assert!(matches!(mempool.add_tx(tx, &state), Err(TxError::BadNonce)));
    }
}

