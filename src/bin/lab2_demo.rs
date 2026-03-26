use toychain::crypto::pubkey_to_address;
use toychain::tx::Transaction;
use toychain::viz;
use ed25519_dalek::Keypair;
use rand::rngs::OsRng;

fn main() {
    viz::start();

    let mut rng = OsRng;
    let alice_kp = Keypair::generate(&mut rng);
    let bob_kp   = Keypair::generate(&mut rng);

    let alice = pubkey_to_address(&alice_kp.public);
    let bob   = pubkey_to_address(&bob_kp.public);

    viz::register(&alice, "Alice");
    viz::register(&bob,   "Bob");

    println!("\nAlice: {}", &alice[..12]);
    println!("Bob:   {}\n", &bob[..12]);

    let mut tx = Transaction::new_unsigned(
        alice.clone(), bob.clone(), 42, 0, alice_kp.public.as_bytes().to_vec(),
    );
    tx.sign(&alice_kp);

    match tx.verify() {
        Ok(_)  => println!("✓ Alice→Bob 42: подпись верна"),
        Err(e) => println!("✗ {}", e),
    }

    let mut fake = tx.clone();
    fake.amount = 9999;
    match fake.verify() {
        Ok(_)  => println!("✓ подделка прошла (это баг!)"),
        Err(e) => println!("✗ попытка изменить сумму: {}", e),
    }

    let mut bad_sig = tx.clone();
    bad_sig.signature[0] ^= 0x01;
    match bad_sig.verify() {
        Ok(_)  => println!("✓ испорченная подпись прошла (это баг!)"),
        Err(e) => println!("✗ испорченная подпись: {}", e),
    }

    let cb = Transaction::new_coinbase(alice.clone(), 50);
    match cb.verify() {
        Ok(_)  => println!("✓ coinbase верифицирован без подписи (это ок)"),
        Err(e) => println!("✗ {}", e),
    }

    viz::flush();
}
