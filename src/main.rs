pub(crate) mod user;

use user::{User, Treasury};

fn main() {
    let mut treasury = Treasury::default();

    // Create two users: Alice (lender) and Bob (borrower)
    let mut alice = User {
        id: 1,
        name: String::from("Alice"),
        ..Default::default()
    };
    let mut bob = User {
        id: 2,
        name: String::from("Bob"),
        ..Default::default()
    };

    // Alice deposits 1000 with fees deducted and enables borrowing.
    alice.deposit_with_fee(1000, &mut treasury, true);
    println!("After Alice's deposit:");
    println!("Alice: {:#?}", alice);
    println!("Treasury: {:#?}", treasury);

    // Bob attempts to borrow 100 from Alice.
    match bob.borrow(&mut alice, 100) {
        Ok(borrowed) => println!("Bob borrowed {} from Alice.", borrowed),
        Err(err) => println!("Borrow failed: {}", err),
    }
    println!("\nAfter borrowing:");
    println!("Alice: {:#?}", alice);
    println!("Bob: {:#?}", bob);

    // Apply interest to Alice's deposit via Treasury.
    match treasury.apply_interest(&mut alice) {
        Ok(interest) => println!("Applied {} interest to Alice's deposit.", interest),
        Err(err) => println!("Interest application failed: {}", err),
    }
    println!("\nFinal Treasury state: {:#?}", treasury);
}
