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
    pub fn on__lparen_A(mut acc: Output) -> Output {
        acc.push('A');
        acc
    }

    /// "(B"
    #[inline(always)]
    pub fn on__lparen_B(mut acc: Output) -> Output {
        acc.push('B');
        acc
    }

    /// "(C"
    #[inline(always)]
    pub fn on__lparen_C(mut acc: Output) -> Output {
        acc.push('C');
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
    pub fn on__lparen_A_comma__space_A(mut acc: Output) -> Output {
        acc.push('A');
        acc
    }

    /// "(A, B"
    pub fn on__lparen_A_comma__space_B(mut acc: Output) -> Output {
        acc.push('B');
        acc
    }

    /// "(A, C"
    pub fn on__lparen_A_comma__space_C(mut acc: Output) -> Output {
        acc.push('C');
        acc
    }

    /// "(A, A, "
    pub fn on__lparen_A_comma__space_A_comma__space_(acc: Output) -> Output {
        acc
    }
}
