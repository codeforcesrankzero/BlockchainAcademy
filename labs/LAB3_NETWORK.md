# Lab 3: Network & P2P Communication

Реализовать P2P сеть для синхронизации блокчейна между узлами.

## Задание

Все тесты находятся в `tests/lab3_tests.rs`.

## Part 1: Basic Networking

Файл: `labs/lab3/simple_network.rs`

Реализовать:

1. `SimpleNetwork::new()` - создать сеть с mpsc каналами
2. `SimpleNetwork::start()` - запустить TcpListener
3. `SimpleNetwork::connect()` - подключиться к peer
4. `handle_connection()` - обработать соединение в цикле

## Part 2: Message Protocol

Реализовать в том же файле:

1. `send_message()` - отправить сообщение
   - Сериализовать в JSON
   - Отправить длину (4 байта big-endian)
   - Отправить данные
   
2. `receive_message()` - получить сообщение
   - Прочитать длину
   - Прочитать данные
   - Десериализовать

3. `SimpleNetwork::broadcast()` - отправить всем peers

## Part 3: Integration

Интегрировать сеть с Chain и Mempool:

1. Обрабатывать NewTransaction
   - Добавить в mempool
   - Relay дальше

2. Обрабатывать NewBlock
   - Валидировать
   - Добавить в chain
   - Очистить mempool

3. Обрабатывать GetBlocks/Blocks
   - Синхронизация цепей

## Запуск тестов

```bash
cargo test lab3 -- --nocapture
```

## Критерии

- test_network_creation
- test_peer_connection
- test_send_receive_message
- test_broadcast
- test_transaction_broadcast
- test_block_broadcast
- test_chain_sync

## Подсказки

send/receive:
```rust
let data = serde_json::to_vec(msg)?;
let len = data.len() as u32;
stream.write_all(&len.to_be_bytes()).await?;
stream.write_all(&data).await?;
stream.flush().await?;
```

handle_connection:
```rust
tokio::select! {
    result = receive_message(&mut stream) => {
        // обработать входящее
    }
    msg = outgoing_rx.recv() => {
        // отправить исходящее
    }
}
```

## Что изучите

- Async programming (tokio)
- TCP networking
- P2P architecture
- Message protocols
