use crate::parser::parse;
use core::iter;

mod prop {
    use super::*;

    quickcheck::quickcheck! {
        fn correct(c: u8) -> bool {
            let valid = if c.is_ascii_digit() {
                Some(c - b'0')
            } else {
                None
            };
            parse(iter::once(c)).ok() == valid
        }
    }
}

mod reduced {
    use super::*;

    #[test]
    fn zero() {
        assert_eq!(parse(b"0".into_iter().copied()), Ok(0))
    }
}
