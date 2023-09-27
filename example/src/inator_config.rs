#![deny(dead_code)]
#![allow(non_snake_case)]

pub mod abc_tuple {
    pub type Output = Vec<char>;

    #[inline(always)]
    pub fn initial() -> Output {
        vec![]
    }

    #[inline(always)]
    pub fn append<T>(mut v: Vec<T>, token: T) -> Vec<T> {
        v.push(token);
        v
    }
}
