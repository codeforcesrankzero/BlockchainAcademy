# Команды

## Графический симулятор

```bash
cargo run --bin sim
```

Открой в браузере: **http://localhost:3000**

Показывает цепочку блоков в реальном времени, балансы участников, события. Кликни на блок — увидишь транзакции внутри. Требует что все лабы реализованы.

---

## Демо по лабам (с визуализацией начиная с lab3)

```bash
cargo run --bin lab1_demo          # лавинный эффект, PoW — только терминал
cargo run --bin lab2_demo          # подписи, подделка — только терминал
cargo run --bin lab3_demo          # первая визуализация → http://localhost:3000
cargo run --bin lab4_demo          # переводы, балансы → http://localhost:3000
```

---

## Тесты

```bash
# все тесты (лабы + юнит-тесты библиотеки)
cargo test

# конкретная лаба
cargo test --test lab1_tests
cargo test --test lab2_tests
cargo test --test lab3_tests
cargo test --test lab4_tests
```

---

## Сброс состояния (если блоки накопились с прошлого запуска)

```bash
pkill -f "target/debug" ; rm -rf blockchain_data
```

---

## Дополнительные демо (сетевые сценарии)

```bash
cargo run --bin attack_51          # симуляция атаки 51%
cargo run --bin network_demo       # несколько нод, P2P синхронизация
cargo run --bin discovery_demo     # peer discovery
cargo run --bin mempool_sync_demo  # синхронизация мемпула между нодами
```
