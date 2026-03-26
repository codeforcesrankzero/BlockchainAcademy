use toychain::crypto::{sha256, meets_difficulty};

fn main() {
    println!("\n=== Лавинный эффект ===\n");

    let pairs = [
        ("hello",  "Hello"),
        ("block1", "block2"),
        ("alice",  "Alice "),
    ];

    for (a, b) in pairs {
        let ha = sha256(a.as_bytes());
        let hb = sha256(b.as_bytes());
        let diff = ha.iter().zip(hb.iter()).filter(|(x, y)| x != y).count();
        println!("  «{}» → {}", a, hex::encode(&ha[..8]));
        println!("  «{}» → {}  ({}/32 байт отличаются)\n", b, hex::encode(&hb[..8]), diff);
    }

    println!("=== Proof of Work: цена сложности ===\n");

    for difficulty in [8u32, 12, 16, 20] {
        let mut nonce: u64 = 0;
        loop {
            let data = format!("block:nonce={}", nonce);
            let hash = sha256(data.as_bytes());
            if meets_difficulty(&hash, difficulty) {
                println!(
                    "  difficulty={:2}  nonce={:>8}  hash={}…",
                    difficulty, nonce, hex::encode(&hash[..4])
                );
                break;
            }
            nonce += 1;
        }
    }

    println!("\nКаждые +4 бита сложности ≈ в 16 раз больше попыток.\n");
}
