use std::io;

type I = u8;

fn main() -> Result<io::Result<()>, inator::IllFormed<I, usize>> {
    inator::digit().to_file("src/parser.rs")
}
