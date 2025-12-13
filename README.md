#### Blockchain academy

Educational blockchain implementation in Rust demonstrating core concepts of cryptocurrency systems.

## Features

- Ed25519 digital signatures
- SHA-256 hashing
- Proof-of-Work mining
- Account-based state model
- Transaction mempool
- Block validation and consensus
- P2P network simulation
- Chain reorganization

## Installation

```bash
git clone <repository>
cd toychain
cargo build --release
```

## Usage

Run simulation:
```bash
cargo run --bin sim
```

Run tests:
```bash
cargo test
```

Available binaries:
- `sim` - blockchain simulator with random transactions
- `network_demo` - P2P network demonstration
- `network_test` - integration tests
- `mempool_sync_demo` - mempool synchronization
- `attack_51` - 51% attack demonstration

## Project Structure

```
toychain/
├── src/
│   ├── crypto.rs      # Cryptographic primitives
│   ├── tx.rs          # Transactions
│   ├── block.rs       # Blocks and mining
│   ├── state.rs       # State management
│   ├── chain.rs       # Blockchain
│   ├── mempool.rs     # Transaction pool
│   ├── network.rs     # P2P networking
│   ├── node.rs        # Full node implementation
│   └── storage.rs     # Persistence
├── tests/             # Integration tests
└── labs/              # Laboratory assignments
```

## API Example

```rust
use toychain::node::Node;

let mut node = Node::new("Node1", "127.0.0.1:8001", 12, 50);
node.start().await?;
node.connect_to("127.0.0.1:8002").await?;

let tx = node.create_transaction(&recipient_addr, 100);
node.broadcast_transaction(tx).await?;

node.mine_block(10).await?;
```

## Architecture

Core components:
- **Transaction**: signed transfers with nonce-based replay protection
- **Block**: container for transactions with Proof-of-Work
- **Chain**: validates and links blocks, maintains state
- **State**: tracks balances and nonces
- **Mempool**: unconfirmed transaction pool
- **Network**: P2P message passing
- **Node**: full node integrating all components

## Configuration

Default parameters:
- Difficulty: 12 bits
- Block reward: 50
- Hash: SHA-256
- Signature: Ed25519

## Testing

```bash
cargo test --lib
cargo test lab1
cargo test lab2
cargo test lab3
```

## Quick Start Recipes

### 1. Local Blockchain Simulation
Run a standalone blockchain without networking:
```bash
cargo run --bin sim
```
Creates local blockchain data, demonstrates mining and state management.

### 2. Interactive Simulation
Configurable multi-user simulation with visualization:
```bash
cargo run --bin interactive_sim
```
Edit `simulation_config.toml` to customize:
- Number of rounds and participants
- Transaction creation probability
- Mining power distribution per user
- Invalid transaction probability

Example configuration for 51% attack scenario:
```toml
[simulation]
duration_rounds = 50
mining_rounds_per_node = [1, 1, 1, 5]  # Last user has 62.5% hash power

[scenarios]
active = "normal"
```

### 3. P2P Network Demo
Run multiple nodes with peer-to-peer networking:

Terminal 1:
```bash
cargo run --bin network_demo
```

### 4. Fork Resolution Demo
See longest-chain consensus in action:
```bash
cargo run --bin discovery_demo
```
Demonstrates:
- Peer discovery through seed nodes
- Fork creation with competing blocks
- Automatic chain reorganization
- Network-wide consensus

### 5. 51% Attack Simulation
Observe attack scenario with malicious majority:
```bash
cargo run --release --bin attack_51
```
Use `--release` for faster mining. Shows double-spend attempt and chain reorganization.

## Documentation

- `ARCHITECTURE.md` - system design and implementation details
- `TUTORIAL.md` - step-by-step guide
- `labs/` - hands-on assignments

## Differences from Production Blockchains

| Aspect | ToyChain | Bitcoin |
|--------|----------|---------|
| PoW difficulty | 12 bits | 70+ bits |
| Network | Simulated | Global P2P |
| Persistence | Optional | Required |
| Fork resolution | Basic | Complete |
| Code size | ~2000 lines | 150,000+ lines |

Educational simplifications for clarity and learning.

## Requirements

- Rust 1.70+


## References

- Bitcoin whitepaper (Satoshi Nakamoto, 2008)
- Mastering Bitcoin (Andreas Antonopoulos)
