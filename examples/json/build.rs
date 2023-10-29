use inator::*;
use std::io;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Stack {}

impl ToSrc for Stack {
    #[inline]
    fn to_src(&self) -> String {
        todo!()
    }
    #[inline]
    fn src_type() -> String {
        "Stack".to_owned()
    }
}

fn main() -> Result<io::Result<()>, IllFormed<char, Stack, usize>> {
    let parser = empty::<char, Stack>();

    parser.determinize().unwrap().to_file("src/parser.rs")
}
