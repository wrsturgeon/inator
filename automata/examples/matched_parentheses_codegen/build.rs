use inator_automata::{dyck_d, IllFormed};
use std::io;

pub fn main() -> Result<io::Result<()>, IllFormed<char, usize>> {
    // Very manually constructed parser recognizing only valid parentheses.
    dyck_d().to_file("src/autogen.rs")
}
