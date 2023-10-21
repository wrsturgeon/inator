use inator_automata::ToSrc;

/// Stack symbols for a parser.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Symbol {
    Paren, // Just one value, but e.g. if we had parens and brackets, we would use two.
}

impl ToSrc for Symbol {
    #[inline]
    fn to_src(&self) -> String {
        Self::src_type()
            + "::"
            + match *self {
                Self::Paren => "Paren",
            }
    }
    #[inline]
    fn src_type() -> String {
        "symbols::Symbol".to_owned()
    }
}
