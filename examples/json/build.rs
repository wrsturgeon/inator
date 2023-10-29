//! Notice that the functional definition looks almost exactly like the formal spec at <https://www.json.org/json-en.html>!

use inator::*;
use std::io;

type I = u8;

fn main() -> Result<io::Result<()>, IllFormed<I, types::Stack, usize>> {
    let empty = toss(b'\x20') | toss(b'\x0A') | toss(b'\x0D') | toss(b'\x09');

    let parser = todo!();

    parser.determinize().unwrap().to_file("src/parser.rs")
}
