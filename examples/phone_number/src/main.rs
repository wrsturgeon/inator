//! See `build.rs` first!

mod autogen; // <-- Automatically generated in `build.rs`! Should be `.gitignore`d.
mod inator_config;

use autogen::{phone_number as parse, phone_number_fuzz as fuzz};

/// Standard U.S. phone number.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PhoneNumber {
    pub area_code: [u8; 3],
    pub number: [u8; 7],
}

fn main() {
    // Print some inputs (guaranteed to be valid and cover the whole range of valid sequences):
    println!("Fuzzing inputs...");
    let mut rng = rand::thread_rng();
    for input in core::iter::from_fn(|| Some(fuzz(&mut rng))).take(32) {
        println!(
            "\"{}\" => {:?}",
            input.iter().copied().collect::<String>(),
            PhoneNumber::from(parse(input.into_iter()).unwrap())
        );
    }
    println!();
}
