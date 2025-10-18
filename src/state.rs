use std::collections::HashMap;

use crate::tx::{Transaction, TxError};

#[derive(Default, Clone, Debug)]
pub struct State {
    pub balances: HashMap<String, u128>,
    pub nonces: HashMap<String, u64>,
}

impl State {
    pub fn balance_of(&self, addr: &str) -> u128 {
        *self.balances.get(addr).unwrap_or(&0)
    }

    pub fn nonce_of(&self, addr: &str) -> u64 {
        *self.nonces.get(addr).unwrap_or(&0)
    }

    pub fn apply_tx(&mut self, tx: &Transaction) -> Result<(), TxError> {
        tx.verify()?;

        if tx.is_coinbase() {
            let bal_to = self.balance_of(&tx.to);
            self.balances.insert(tx.to.clone(), bal_to + tx.amount as u128);
            return Ok(());
        }

        let expected = self.nonce_of(&tx.from);
        if tx.nonce != expected {
            return Err(TxError::BadNonce);
        }
        let bal_from = self.balance_of(&tx.from);
        if bal_from < tx.amount as u128 {
            return Err(TxError::InsufficientFunds);
        }
        self.balances.insert(tx.from.clone(), bal_from - tx.amount as u128);
        let bal_to = self.balance_of(&tx.to);
        self.balances.insert(tx.to.clone(), bal_to + tx.amount as u128);
        self.nonces.insert(tx.from.clone(), expected + 1);
        Ok(())
    }

    pub fn apply_block(&mut self, block: &crate::block::Block) -> Result<(), TxError> {
        for tx in &block.txs {
            self.apply_tx(tx)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;
    use ed25519_dalek::Keypair;

    #[test]
    fn apply_tx_ok_and_counters() {
        let mut rng = OsRng;
        let kp = Keypair::generate(&mut rng);
        let from = crate::crypto::pubkey_to_address(&kp.public);

        let mut st = State::default();
        st.balances.insert(from.clone(), 100);

        let mut tx = Transaction::new_unsigned(
            from.clone(),
            from.clone(),
            40,
            0,
            kp.public.as_bytes().to_vec(),
        );
        tx.sign(&kp);

        st.apply_tx(&tx).unwrap();
        assert_eq!(st.balance_of(&from), 100);
        assert_eq!(st.nonce_of(&from), 1);
    }

    #[test]
    fn insufficient_funds() {
        let mut rng = OsRng;
        let kp = Keypair::generate(&mut rng);
        let from = crate::crypto::pubkey_to_address(&kp.public);

        let mut st = State::default();
        st.balances.insert(from.clone(), 10);

        let mut tx = Transaction::new_unsigned(
            from.clone(), from.clone(), 11, 0, kp.public.as_bytes().to_vec()
        );
        tx.sign(&kp);

        let err = st.apply_tx(&tx).unwrap_err();
        matches!(err, crate::tx::TxError::InsufficientFunds);
    }
}
