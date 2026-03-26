#![allow(dead_code)]

#[path = "../../labs/lab3/block.rs"]
mod student;
use student::Block;

use toychain::crypto::meets_difficulty;

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
}
use toychain::tx::Transaction;
use toychain::viz::{self, VizEvent, TxInfo};
use std::thread::sleep;
use std::time::Duration;

fn main() {
    viz::start();
    println!("\nОткрой http://localhost:3000\n");

    println!("=== Цена сложности ===\n");
    for difficulty in [8u32, 12, 16, 20] {
        let b = Block {
            height: 1, timestamp: now(),
            prev_hash: [0u8; 32], nonce: 0, difficulty,
            txs: vec![],
        };
        let mined = b.mine(); // ← КОД СТУДЕНТА
        println!(
            "  difficulty={:2}  nonce={:>8}  hash={}…  meets={}",
            difficulty, mined.nonce,
            hex::encode(&mined.header_hash()[..4]),
            meets_difficulty(&mined.header_hash(), difficulty)
        );
    }
    println!();

    println!("=== Цепочка в браузере (difficulty=12) ===\n");

    let genesis = Block {
        height: 0, timestamp: now(),
        prev_hash: [0u8; 32], nonce: 0, difficulty: 12, txs: vec![],
    }.mine(); // ← КОД СТУДЕНТА

    viz::emit(VizEvent::Genesis {
        hash_prefix: hex::encode(&genesis.header_hash()[..4]),
    });

    let mut prev_hash = genesis.header_hash();

    for i in 1u64..=5 {
        sleep(Duration::from_millis(700));

        let coinbase = Transaction::new_coinbase("demo_miner".to_string(), 50);
        let b = Block {
            height: i, timestamp: now(),
            prev_hash, nonce: 0, difficulty: 12,
            txs: vec![coinbase],
        }.mine(); // ← КОД СТУДЕНТА

        let hash = hex::encode(&b.header_hash()[..4]);
        prev_hash = b.header_hash();

        viz::emit(VizEvent::BlockMined {
            height: i, miner: "demo_miner".to_string(),
            tx_count: b.txs.len(), nonce: b.nonce,
        });
        viz::emit(VizEvent::BlockAdded {
            height: i,
            hash_prefix: hash.clone(),
            miner: "demo_miner".to_string(),
            txs: vec![TxInfo {
                from: String::new(), to: "demo_miner".to_string(),
                amount: 50, is_coinbase: true,
            }],
        });
        viz::emit(VizEvent::BalanceChange {
            addr: "demo_miner".to_string(),
            old_bal: ((i - 1) * 50) as u128,
            new_bal: (i * 50) as u128,
        });

        println!("  Блок #{i}  nonce={}  hash={hash}…", b.nonce);
    }

    viz::flush();
    sleep(Duration::from_secs(1));
}
