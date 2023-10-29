use inator::*;
use std::io;

type I = u8;

fn main() -> Result<io::Result<()>, IllFormed<I, types::Stack, usize>> {
    let parser = empty::<I, types::Stack>();

    parser.determinize().unwrap().to_file("src/parser.rs")
}
