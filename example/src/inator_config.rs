#![allow(non_snake_case)]

pub mod abc_tuple {
    pub type Output = Vec<char>;

    #[inline(always)]
    pub fn initial() -> Output {
        vec![]
    }

    /// "("
    #[inline(always)]
    pub fn on__lparen_(acc: Output) -> Output {
        acc
    }

    /// "()"
    #[inline(always)]
    pub fn on__lparen__rparen_(acc: Output) -> Output {
        acc
    }

    /// "(A"
    #[inline(always)]
    pub fn on__lparen_A(
        mut acc: Output,
        alt: crate::autogen::abc_tuple_states::Alternates__lparen_A,
    ) -> Output {
        acc.push(alt.into());
        acc
    }

    /// "(A, A)"
    #[inline(always)]
    pub fn on__lparen_A_comma__space_A_rparen_(acc: Output) -> Output {
        acc
    }

    /// "(A, A,"
    #[inline(always)]
    pub fn on__lparen_A_comma__space_A_comma_(acc: Output) -> Output {
        acc
    }

    /// "(A, "
    #[inline(always)]
    pub fn on__lparen_A_comma__space_(acc: Output) -> Output {
        acc
    }

    /// "(A,)"
    #[inline(always)]
    pub fn on__lparen_A_comma__rparen_(acc: Output) -> Output {
        acc
    }

    /// "(A,"
    #[inline(always)]
    pub fn on__lparen_A_comma_(acc: Output) -> Output {
        acc
    }

    /// "(A, A"
    pub fn on__lparen_A_comma__space_A(
        mut acc: Output,
        alt: crate::autogen::abc_tuple_states::Alternates__lparen_A_comma__space_A,
    ) -> Output {
        acc.push(alt.into());
        acc
    }

    /// "(A, A, "
    pub fn on__lparen_A_comma__space_A_comma__space_(acc: Output) -> Output {
        acc
    }
}
