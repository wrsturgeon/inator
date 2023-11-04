use core::iter;

quickcheck::quickcheck! {
    fn correct(i: u8) -> bool {
        let d = b'0' + (i % 10);
        crate::parser::parse(iter::once(d)) == Ok(d)
    }
}
