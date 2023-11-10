use crate::parser::parse;
use core::iter;

mod prop {
    use super::*;

    quickcheck::quickcheck! {
        fn sensitivity(i: usize) -> bool {
            let s = i.to_string();
            parse(s.bytes()) == Ok(i)
        }

        fn specificity(s: String) -> bool {
            let valid: Option<usize> = s.parse().ok();
            parse(s.bytes()).ok() == valid
        }
    }
}

mod reduced {
    use super::*;
}
