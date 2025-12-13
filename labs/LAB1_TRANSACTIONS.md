# Lab 1: Transactions and Blocks

Реализовать базовые структуры блокчейна - транзакции и блоки.

## Задание

Все тесты находятся в `tests/lab1_tests.rs`. Ваша задача - сделать их зелеными.

## Part 1: Transaction

Файл: `labs/lab1/transaction.rs`

Реализовать:

1. `Transaction::new_unsigned()` - создать неподписанную транзакцию
2. `Transaction::sign()` - подписать транзакцию используя ed25519-dalek
3. `Transaction::verify()` - проверить подпись
4. `Transaction::hash()` - вычислить хеш транзакции

## Part 2: Block

Файл: `labs/lab1/block.rs`

Реализовать:

1. `Block::txs_root()` - вычислить Merkle root транзакций (упрощенный)
2. `Block::header_hash()` - вычислить хеш заголовка блока
3. `Block::mine()` - намайнить блок (Proof of Work)

## Запуск тестов

```bash
cargo test lab1 -- --nocapture
```

## Критерии

Part 1:
- test_transaction_creation
- test_transaction_signing  
- test_transaction_verification
- test_transaction_invalid_signature
- test_transaction_hash

Part 2:
- test_block_creation
- test_block_txs_root
- test_block_header_hash
- test_block_mining

## Подсказки

Transaction::sign():
```rust
let message = self.bytes_for_signing();
let signature = keypair.sign(&message);
self.signature = signature.to_bytes().to_vec();
```

Transaction::verify():
```rust
tx.verify()?;
let public_key = PublicKey::from_bytes(&self.pubkey)?;
let signature = Signature::from_bytes(&self.signature)?;
public_key.verify(&message, &signature)?;
```

Block::mine():
```rust
loop {
    let hash = self.header_hash();
    if meets_difficulty(&hash, self.difficulty) {
        return self;
    }
    self.nonce += 1;
}
```

## Что изучите

- Цифровые подписи (Ed25519)
- Хеширование (SHA-256)
- Proof of Work
- Merkle root
