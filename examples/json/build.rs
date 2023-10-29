use inator::*;
use std::io;

fn main() -> Result<io::Result<()>, IllFormed<char, types::Stack, usize>> {
    let parser = empty::<char, types::Stack>();

    parser.determinize().unwrap().to_file("src/parser.rs")
}
