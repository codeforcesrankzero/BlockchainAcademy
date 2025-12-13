# Lab 2: Chain and State Management

Реализовать управление цепью блоков и состоянием балансов.

## Задание

Все тесты находятся в `tests/lab2_tests.rs`.

## Part 1: State

Файл: `labs/lab2/state.rs`

Реализовать:

1. `State::balance_of()` - получить баланс адреса (0 если нет)
2. `State::nonce_of()` - получить nonce адреса (0 если нет)
3. `State::apply_tx()` - применить транзакцию к состоянию
   - Для coinbase: добавить amount к балансу
   - Для обычных: проверить подпись, nonce, баланс, обновить балансы и nonce
4. `State::apply_block()` - применить все транзакции блока

## Part 2: Chain

Файл: `labs/lab2/chain.rs`

Реализовать:

1. `Chain::genesis()` - создать genesis блок
2. `Chain::tip_hash()` - хеш последнего блока
3. `Chain::height()` - высота цепи
4. `Chain::add_block()` - валидировать и добавить блок
   - Проверить height, prev_hash, timestamp, PoW
   - Проверить coinbase (первая tx, правильная сумма)
   - Применить транзакции
5. `Chain::mine_block()` - намайнить новый блок с транзакциями

## Запуск тестов

```bash
cargo test lab2 -- --nocapture
```

## Критерии

Part 1:
- test_state_balance
- test_state_nonce
- test_apply_transaction
- test_apply_coinbase
- test_insufficient_funds
- test_bad_nonce

Part 2:
- test_genesis
- test_chain_height
- test_add_valid_block
- test_reject_wrong_height
- test_reject_wrong_prev_hash
- test_reject_bad_pow
- test_mine_block

## Подсказки

State::apply_tx():
```rust
if tx.is_coinbase() {
    let balance = self.balance_of(&tx.to);
    self.balances.insert(tx.to.clone(), balance + tx.amount as u128);
    return Ok(());
}

tx.verify()?;
if tx.nonce != self.nonce_of(&tx.from) {
    return Err(TxError::BadNonce);
}
```

Chain::add_block():
```rust
if block.height != expected_height {
    return Err(BlockError::BadHeight);
}
if block.prev_hash != self.tip_hash() {
    return Err(BlockError::PrevHashMismatch);
}
if !meets_difficulty(&block.header_hash(), block.difficulty) {
    return Err(BlockError::BadPoW);
}
```

## Что изучите

- State management
- Chain validation
- Error handling
- Consensus rules
