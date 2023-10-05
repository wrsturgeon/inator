//! Boilerplate to generate property-test input.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Alphabet {
    A,
    B,
    C,
}

impl From<Alphabet> for char {
    #[inline]
    fn from(a: Alphabet) -> Self {
        match a {
            Alphabet::A => 'A',
            Alphabet::B => 'B',
            Alphabet::C => 'C',
        }
    }
}

impl quickcheck::Arbitrary for Alphabet {
    #[inline]
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        *g.choose(&[Self::A, Self::B, Self::C]).unwrap()
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match *self {
            Self::A => Box::new(core::iter::empty()),
            Self::B => Box::new(core::iter::once(Self::A)),
            Self::C => Box::new([Self::A, Self::B].into_iter()),
        }
    }
}

pub fn mk_input(v: &[Alphabet]) -> String {
    v.split_first().map_or_else(
        || "()".to_owned(),
        |(head, tail)| {
            if tail.is_empty() {
                format!("({},)", char::from(*head))
            } else {
                format!(
                    "({}{})",
                    char::from(*head),
                    tail.iter().fold(String::new(), |acc, c| acc
                        + &format!(", {}", char::from(*c)))
                )
            }
        },
    )
}

pub fn roundtrip(g: &mut quickcheck::Gen) {
    let v = <Vec<Alphabet> as quickcheck::Arbitrary>::arbitrary(g);
    let input = mk_input(&v);
    let parsed = crate::parse(input.chars()).unwrap();
    println!("\"{input}\" -> {parsed:?}");
    assert_eq!(parsed, v.into_iter().map(char::from).collect::<Vec<_>>());
}
