#[inline(always)]
pub fn phone_number<I: Iterator<Item = char>>(
    i: I,
) -> Option<crate::inator_config::phone_number::Output> {
    phone_number_states::s0(i, crate::inator_config::phone_number::initial())
}
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[allow(unused_parens)]
#[allow(unused_variables)]
pub(crate) mod phone_number_states {
    #[inline]
    ///Minimal input to reach this state: [this is the initial state]
    pub fn s0<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::phone_number::Output,
    ) -> Option<crate::inator_config::phone_number::Output> {
        match i.next() {
            Some(token @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                s1(i, crate::inator_config::phone_number::digit(acc, token))
            }
            Some(token @ ('(')) => s4(i, acc),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0'
    pub fn s1<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::phone_number::Output,
    ) -> Option<crate::inator_config::phone_number::Output> {
        match i.next() {
            Some(token @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                s2(i, crate::inator_config::phone_number::digit(acc, token))
            }
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0'
    pub fn s2<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::phone_number::Output,
    ) -> Option<crate::inator_config::phone_number::Output> {
        match i.next() {
            Some(token @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                s3(i, crate::inator_config::phone_number::digit(acc, token))
            }
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0'
    pub fn s3<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::phone_number::Output,
    ) -> Option<crate::inator_config::phone_number::Output> {
        match i.next() {
            Some(token @ (' ')) => s8(i, acc),
            Some(token @ ('.')) => s17(i, acc),
            Some(token @ ('-')) => s21(i, acc),
            Some(token @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                s25(i, crate::inator_config::phone_number::digit(acc, token))
            }
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '('
    pub fn s4<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::phone_number::Output,
    ) -> Option<crate::inator_config::phone_number::Output> {
        match i.next() {
            Some(token @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                s5(i, crate::inator_config::phone_number::digit(acc, token))
            }
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '(' -> '0'
    pub fn s5<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::phone_number::Output,
    ) -> Option<crate::inator_config::phone_number::Output> {
        match i.next() {
            Some(token @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                s6(i, crate::inator_config::phone_number::digit(acc, token))
            }
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '(' -> '0' -> '0'
    pub fn s6<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::phone_number::Output,
    ) -> Option<crate::inator_config::phone_number::Output> {
        match i.next() {
            Some(token @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                s7(i, crate::inator_config::phone_number::digit(acc, token))
            }
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '(' -> '0' -> '0' -> '0'
    pub fn s7<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::phone_number::Output,
    ) -> Option<crate::inator_config::phone_number::Output> {
        match i.next() {
            Some(token @ (')')) => s3(i, acc),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> ' '
    pub fn s8<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::phone_number::Output,
    ) -> Option<crate::inator_config::phone_number::Output> {
        match i.next() {
            Some(token @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                s9(i, crate::inator_config::phone_number::digit(acc, token))
            }
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> ' ' -> '0'
    pub fn s9<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::phone_number::Output,
    ) -> Option<crate::inator_config::phone_number::Output> {
        match i.next() {
            Some(token @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                s10(i, crate::inator_config::phone_number::digit(acc, token))
            }
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> ' ' -> '0' -> '0'
    pub fn s10<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::phone_number::Output,
    ) -> Option<crate::inator_config::phone_number::Output> {
        match i.next() {
            Some(token @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                s11(i, crate::inator_config::phone_number::digit(acc, token))
            }
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> ' ' -> '0' -> '0' -> '0'
    pub fn s11<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::phone_number::Output,
    ) -> Option<crate::inator_config::phone_number::Output> {
        match i.next() {
            Some(token @ (' ')) => s12(i, acc),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '0' -> '0' -> '0'
    pub fn s12<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::phone_number::Output,
    ) -> Option<crate::inator_config::phone_number::Output> {
        match i.next() {
            Some(token @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                s13(i, crate::inator_config::phone_number::digit(acc, token))
            }
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '0' -> '0' -> '0' -> '0'
    pub fn s13<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::phone_number::Output,
    ) -> Option<crate::inator_config::phone_number::Output> {
        match i.next() {
            Some(token @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                s14(i, crate::inator_config::phone_number::digit(acc, token))
            }
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '0' -> '0' -> '0' -> '0' -> '0'
    pub fn s14<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::phone_number::Output,
    ) -> Option<crate::inator_config::phone_number::Output> {
        match i.next() {
            Some(token @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                s15(i, crate::inator_config::phone_number::digit(acc, token))
            }
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '0' -> '0' -> '0' -> '0' -> '0' -> '0'
    pub fn s15<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::phone_number::Output,
    ) -> Option<crate::inator_config::phone_number::Output> {
        match i.next() {
            Some(token @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                s16(i, crate::inator_config::phone_number::digit(acc, token))
            }
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '0' -> '0' -> '0' -> '0' -> '0' -> '0' -> '0'
    pub fn s16<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::phone_number::Output,
    ) -> Option<crate::inator_config::phone_number::Output> {
        match i.next() {
            None => Some(acc),
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '.'
    pub fn s17<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::phone_number::Output,
    ) -> Option<crate::inator_config::phone_number::Output> {
        match i.next() {
            Some(token @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                s18(i, crate::inator_config::phone_number::digit(acc, token))
            }
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '.' -> '0'
    pub fn s18<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::phone_number::Output,
    ) -> Option<crate::inator_config::phone_number::Output> {
        match i.next() {
            Some(token @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                s19(i, crate::inator_config::phone_number::digit(acc, token))
            }
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '.' -> '0' -> '0'
    pub fn s19<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::phone_number::Output,
    ) -> Option<crate::inator_config::phone_number::Output> {
        match i.next() {
            Some(token @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                s20(i, crate::inator_config::phone_number::digit(acc, token))
            }
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '.' -> '0' -> '0' -> '0'
    pub fn s20<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::phone_number::Output,
    ) -> Option<crate::inator_config::phone_number::Output> {
        match i.next() {
            Some(token @ ('.')) => s12(i, acc),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '-'
    pub fn s21<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::phone_number::Output,
    ) -> Option<crate::inator_config::phone_number::Output> {
        match i.next() {
            Some(token @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                s22(i, crate::inator_config::phone_number::digit(acc, token))
            }
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '-' -> '0'
    pub fn s22<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::phone_number::Output,
    ) -> Option<crate::inator_config::phone_number::Output> {
        match i.next() {
            Some(token @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                s23(i, crate::inator_config::phone_number::digit(acc, token))
            }
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '-' -> '0' -> '0'
    pub fn s23<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::phone_number::Output,
    ) -> Option<crate::inator_config::phone_number::Output> {
        match i.next() {
            Some(token @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                s24(i, crate::inator_config::phone_number::digit(acc, token))
            }
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '-' -> '0' -> '0' -> '0'
    pub fn s24<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::phone_number::Output,
    ) -> Option<crate::inator_config::phone_number::Output> {
        match i.next() {
            Some(token @ ('-')) => s12(i, acc),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '0'
    pub fn s25<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::phone_number::Output,
    ) -> Option<crate::inator_config::phone_number::Output> {
        match i.next() {
            Some(token @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                s26(i, crate::inator_config::phone_number::digit(acc, token))
            }
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '0' -> '0'
    pub fn s26<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::phone_number::Output,
    ) -> Option<crate::inator_config::phone_number::Output> {
        match i.next() {
            Some(token @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                s12(i, crate::inator_config::phone_number::digit(acc, token))
            }
            None => None,
            _ => None,
        }
    }
}
#[inline]
pub fn phone_number_fuzz<R: rand::Rng>(r: &mut R) -> Vec<char> {
    let mut v = phone_number_fuzz_states::s16(r, vec![]);
    v.reverse();
    v
}
#[allow(non_snake_case)]
#[allow(unused_mut)]
mod phone_number_fuzz_states {
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '0' -> '0' -> '0' -> '0' -> '0' -> '0' -> '0'
    pub fn s0<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        if r.gen() {
            return acc;
        }
        s16(r, vec![])
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '0' -> '0' -> '0' -> '0' -> '0' -> '0'
    pub fn s1<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..10) {
            0 => {
                acc.push('0');
                s0(r, acc)
            }
            1 => {
                acc.push('1');
                s0(r, acc)
            }
            2 => {
                acc.push('2');
                s0(r, acc)
            }
            3 => {
                acc.push('3');
                s0(r, acc)
            }
            4 => {
                acc.push('4');
                s0(r, acc)
            }
            5 => {
                acc.push('5');
                s0(r, acc)
            }
            6 => {
                acc.push('6');
                s0(r, acc)
            }
            7 => {
                acc.push('7');
                s0(r, acc)
            }
            8 => {
                acc.push('8');
                s0(r, acc)
            }
            9 => {
                acc.push('9');
                s0(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '0' -> '0' -> '0' -> '0' -> '0'
    pub fn s2<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..10) {
            0 => {
                acc.push('0');
                s1(r, acc)
            }
            1 => {
                acc.push('1');
                s1(r, acc)
            }
            2 => {
                acc.push('2');
                s1(r, acc)
            }
            3 => {
                acc.push('3');
                s1(r, acc)
            }
            4 => {
                acc.push('4');
                s1(r, acc)
            }
            5 => {
                acc.push('5');
                s1(r, acc)
            }
            6 => {
                acc.push('6');
                s1(r, acc)
            }
            7 => {
                acc.push('7');
                s1(r, acc)
            }
            8 => {
                acc.push('8');
                s1(r, acc)
            }
            9 => {
                acc.push('9');
                s1(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '0' -> '0' -> '0' -> '0'
    pub fn s3<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..11) {
            0 => {
                acc.push(')');
                s7(r, acc)
            }
            1 => {
                acc.push('0');
                s2(r, acc)
            }
            2 => {
                acc.push('1');
                s2(r, acc)
            }
            3 => {
                acc.push('2');
                s2(r, acc)
            }
            4 => {
                acc.push('3');
                s2(r, acc)
            }
            5 => {
                acc.push('4');
                s2(r, acc)
            }
            6 => {
                acc.push('5');
                s2(r, acc)
            }
            7 => {
                acc.push('6');
                s2(r, acc)
            }
            8 => {
                acc.push('7');
                s2(r, acc)
            }
            9 => {
                acc.push('8');
                s2(r, acc)
            }
            10 => {
                acc.push('9');
                s2(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '0' -> '0' -> '0' -> '0' -> ')' -> '0' -> '0' -> '0'
    pub fn s4<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        acc.push('(');
        s0(r, acc)
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '0' -> '0' -> '0' -> '0' -> ')' -> '0' -> '0'
    pub fn s5<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..10) {
            0 => {
                acc.push('0');
                s4(r, acc)
            }
            1 => {
                acc.push('1');
                s4(r, acc)
            }
            2 => {
                acc.push('2');
                s4(r, acc)
            }
            3 => {
                acc.push('3');
                s4(r, acc)
            }
            4 => {
                acc.push('4');
                s4(r, acc)
            }
            5 => {
                acc.push('5');
                s4(r, acc)
            }
            6 => {
                acc.push('6');
                s4(r, acc)
            }
            7 => {
                acc.push('7');
                s4(r, acc)
            }
            8 => {
                acc.push('8');
                s4(r, acc)
            }
            9 => {
                acc.push('9');
                s4(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '0' -> '0' -> '0' -> '0' -> ')' -> '0'
    pub fn s6<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..10) {
            0 => {
                acc.push('0');
                s5(r, acc)
            }
            1 => {
                acc.push('1');
                s5(r, acc)
            }
            2 => {
                acc.push('2');
                s5(r, acc)
            }
            3 => {
                acc.push('3');
                s5(r, acc)
            }
            4 => {
                acc.push('4');
                s5(r, acc)
            }
            5 => {
                acc.push('5');
                s5(r, acc)
            }
            6 => {
                acc.push('6');
                s5(r, acc)
            }
            7 => {
                acc.push('7');
                s5(r, acc)
            }
            8 => {
                acc.push('8');
                s5(r, acc)
            }
            9 => {
                acc.push('9');
                s5(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '0' -> '0' -> '0' -> '0' -> ')'
    pub fn s7<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..10) {
            0 => {
                acc.push('0');
                s6(r, acc)
            }
            1 => {
                acc.push('1');
                s6(r, acc)
            }
            2 => {
                acc.push('2');
                s6(r, acc)
            }
            3 => {
                acc.push('3');
                s6(r, acc)
            }
            4 => {
                acc.push('4');
                s6(r, acc)
            }
            5 => {
                acc.push('5');
                s6(r, acc)
            }
            6 => {
                acc.push('6');
                s6(r, acc)
            }
            7 => {
                acc.push('7');
                s6(r, acc)
            }
            8 => {
                acc.push('8');
                s6(r, acc)
            }
            9 => {
                acc.push('9');
                s6(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '0' -> ' ' -> '0' -> '0' -> '0'
    pub fn s8<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        acc.push(' ');
        s3(r, acc)
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '0' -> ' ' -> '0' -> '0'
    pub fn s9<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..10) {
            0 => {
                acc.push('0');
                s8(r, acc)
            }
            1 => {
                acc.push('1');
                s8(r, acc)
            }
            2 => {
                acc.push('2');
                s8(r, acc)
            }
            3 => {
                acc.push('3');
                s8(r, acc)
            }
            4 => {
                acc.push('4');
                s8(r, acc)
            }
            5 => {
                acc.push('5');
                s8(r, acc)
            }
            6 => {
                acc.push('6');
                s8(r, acc)
            }
            7 => {
                acc.push('7');
                s8(r, acc)
            }
            8 => {
                acc.push('8');
                s8(r, acc)
            }
            9 => {
                acc.push('9');
                s8(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '0' -> ' ' -> '0'
    pub fn s10<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..10) {
            0 => {
                acc.push('0');
                s9(r, acc)
            }
            1 => {
                acc.push('1');
                s9(r, acc)
            }
            2 => {
                acc.push('2');
                s9(r, acc)
            }
            3 => {
                acc.push('3');
                s9(r, acc)
            }
            4 => {
                acc.push('4');
                s9(r, acc)
            }
            5 => {
                acc.push('5');
                s9(r, acc)
            }
            6 => {
                acc.push('6');
                s9(r, acc)
            }
            7 => {
                acc.push('7');
                s9(r, acc)
            }
            8 => {
                acc.push('8');
                s9(r, acc)
            }
            9 => {
                acc.push('9');
                s9(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '0' -> ' '
    pub fn s11<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..10) {
            0 => {
                acc.push('0');
                s10(r, acc)
            }
            1 => {
                acc.push('1');
                s10(r, acc)
            }
            2 => {
                acc.push('2');
                s10(r, acc)
            }
            3 => {
                acc.push('3');
                s10(r, acc)
            }
            4 => {
                acc.push('4');
                s10(r, acc)
            }
            5 => {
                acc.push('5');
                s10(r, acc)
            }
            6 => {
                acc.push('6');
                s10(r, acc)
            }
            7 => {
                acc.push('7');
                s10(r, acc)
            }
            8 => {
                acc.push('8');
                s10(r, acc)
            }
            9 => {
                acc.push('9');
                s10(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '0'
    pub fn s12<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..13) {
            0 => {
                acc.push(' ');
                s11(r, acc)
            }
            1 => {
                acc.push('-');
                s24(r, acc)
            }
            2 => {
                acc.push('.');
                s20(r, acc)
            }
            3 => {
                acc.push('0');
                s26(r, acc)
            }
            4 => {
                acc.push('1');
                s26(r, acc)
            }
            5 => {
                acc.push('2');
                s26(r, acc)
            }
            6 => {
                acc.push('3');
                s26(r, acc)
            }
            7 => {
                acc.push('4');
                s26(r, acc)
            }
            8 => {
                acc.push('5');
                s26(r, acc)
            }
            9 => {
                acc.push('6');
                s26(r, acc)
            }
            10 => {
                acc.push('7');
                s26(r, acc)
            }
            11 => {
                acc.push('8');
                s26(r, acc)
            }
            12 => {
                acc.push('9');
                s26(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0'
    pub fn s13<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..10) {
            0 => {
                acc.push('0');
                s12(r, acc)
            }
            1 => {
                acc.push('1');
                s12(r, acc)
            }
            2 => {
                acc.push('2');
                s12(r, acc)
            }
            3 => {
                acc.push('3');
                s12(r, acc)
            }
            4 => {
                acc.push('4');
                s12(r, acc)
            }
            5 => {
                acc.push('5');
                s12(r, acc)
            }
            6 => {
                acc.push('6');
                s12(r, acc)
            }
            7 => {
                acc.push('7');
                s12(r, acc)
            }
            8 => {
                acc.push('8');
                s12(r, acc)
            }
            9 => {
                acc.push('9');
                s12(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0'
    pub fn s14<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..10) {
            0 => {
                acc.push('0');
                s13(r, acc)
            }
            1 => {
                acc.push('1');
                s13(r, acc)
            }
            2 => {
                acc.push('2');
                s13(r, acc)
            }
            3 => {
                acc.push('3');
                s13(r, acc)
            }
            4 => {
                acc.push('4');
                s13(r, acc)
            }
            5 => {
                acc.push('5');
                s13(r, acc)
            }
            6 => {
                acc.push('6');
                s13(r, acc)
            }
            7 => {
                acc.push('7');
                s13(r, acc)
            }
            8 => {
                acc.push('8');
                s13(r, acc)
            }
            9 => {
                acc.push('9');
                s13(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0'
    pub fn s15<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..10) {
            0 => {
                acc.push('0');
                s14(r, acc)
            }
            1 => {
                acc.push('1');
                s14(r, acc)
            }
            2 => {
                acc.push('2');
                s14(r, acc)
            }
            3 => {
                acc.push('3');
                s14(r, acc)
            }
            4 => {
                acc.push('4');
                s14(r, acc)
            }
            5 => {
                acc.push('5');
                s14(r, acc)
            }
            6 => {
                acc.push('6');
                s14(r, acc)
            }
            7 => {
                acc.push('7');
                s14(r, acc)
            }
            8 => {
                acc.push('8');
                s14(r, acc)
            }
            9 => {
                acc.push('9');
                s14(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: [this is the initial state]
    pub fn s16<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..10) {
            0 => {
                acc.push('0');
                s15(r, acc)
            }
            1 => {
                acc.push('1');
                s15(r, acc)
            }
            2 => {
                acc.push('2');
                s15(r, acc)
            }
            3 => {
                acc.push('3');
                s15(r, acc)
            }
            4 => {
                acc.push('4');
                s15(r, acc)
            }
            5 => {
                acc.push('5');
                s15(r, acc)
            }
            6 => {
                acc.push('6');
                s15(r, acc)
            }
            7 => {
                acc.push('7');
                s15(r, acc)
            }
            8 => {
                acc.push('8');
                s15(r, acc)
            }
            9 => {
                acc.push('9');
                s15(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '0' -> '.' -> '0' -> '0' -> '0'
    pub fn s17<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        acc.push('.');
        s3(r, acc)
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '0' -> '.' -> '0' -> '0'
    pub fn s18<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..10) {
            0 => {
                acc.push('0');
                s17(r, acc)
            }
            1 => {
                acc.push('1');
                s17(r, acc)
            }
            2 => {
                acc.push('2');
                s17(r, acc)
            }
            3 => {
                acc.push('3');
                s17(r, acc)
            }
            4 => {
                acc.push('4');
                s17(r, acc)
            }
            5 => {
                acc.push('5');
                s17(r, acc)
            }
            6 => {
                acc.push('6');
                s17(r, acc)
            }
            7 => {
                acc.push('7');
                s17(r, acc)
            }
            8 => {
                acc.push('8');
                s17(r, acc)
            }
            9 => {
                acc.push('9');
                s17(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '0' -> '.' -> '0'
    pub fn s19<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..10) {
            0 => {
                acc.push('0');
                s18(r, acc)
            }
            1 => {
                acc.push('1');
                s18(r, acc)
            }
            2 => {
                acc.push('2');
                s18(r, acc)
            }
            3 => {
                acc.push('3');
                s18(r, acc)
            }
            4 => {
                acc.push('4');
                s18(r, acc)
            }
            5 => {
                acc.push('5');
                s18(r, acc)
            }
            6 => {
                acc.push('6');
                s18(r, acc)
            }
            7 => {
                acc.push('7');
                s18(r, acc)
            }
            8 => {
                acc.push('8');
                s18(r, acc)
            }
            9 => {
                acc.push('9');
                s18(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '0' -> '.'
    pub fn s20<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..10) {
            0 => {
                acc.push('0');
                s19(r, acc)
            }
            1 => {
                acc.push('1');
                s19(r, acc)
            }
            2 => {
                acc.push('2');
                s19(r, acc)
            }
            3 => {
                acc.push('3');
                s19(r, acc)
            }
            4 => {
                acc.push('4');
                s19(r, acc)
            }
            5 => {
                acc.push('5');
                s19(r, acc)
            }
            6 => {
                acc.push('6');
                s19(r, acc)
            }
            7 => {
                acc.push('7');
                s19(r, acc)
            }
            8 => {
                acc.push('8');
                s19(r, acc)
            }
            9 => {
                acc.push('9');
                s19(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '0' -> '-' -> '0' -> '0' -> '0'
    pub fn s21<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        acc.push('-');
        s3(r, acc)
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '0' -> '-' -> '0' -> '0'
    pub fn s22<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..10) {
            0 => {
                acc.push('0');
                s21(r, acc)
            }
            1 => {
                acc.push('1');
                s21(r, acc)
            }
            2 => {
                acc.push('2');
                s21(r, acc)
            }
            3 => {
                acc.push('3');
                s21(r, acc)
            }
            4 => {
                acc.push('4');
                s21(r, acc)
            }
            5 => {
                acc.push('5');
                s21(r, acc)
            }
            6 => {
                acc.push('6');
                s21(r, acc)
            }
            7 => {
                acc.push('7');
                s21(r, acc)
            }
            8 => {
                acc.push('8');
                s21(r, acc)
            }
            9 => {
                acc.push('9');
                s21(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '0' -> '-' -> '0'
    pub fn s23<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..10) {
            0 => {
                acc.push('0');
                s22(r, acc)
            }
            1 => {
                acc.push('1');
                s22(r, acc)
            }
            2 => {
                acc.push('2');
                s22(r, acc)
            }
            3 => {
                acc.push('3');
                s22(r, acc)
            }
            4 => {
                acc.push('4');
                s22(r, acc)
            }
            5 => {
                acc.push('5');
                s22(r, acc)
            }
            6 => {
                acc.push('6');
                s22(r, acc)
            }
            7 => {
                acc.push('7');
                s22(r, acc)
            }
            8 => {
                acc.push('8');
                s22(r, acc)
            }
            9 => {
                acc.push('9');
                s22(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '0' -> '-'
    pub fn s24<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..10) {
            0 => {
                acc.push('0');
                s23(r, acc)
            }
            1 => {
                acc.push('1');
                s23(r, acc)
            }
            2 => {
                acc.push('2');
                s23(r, acc)
            }
            3 => {
                acc.push('3');
                s23(r, acc)
            }
            4 => {
                acc.push('4');
                s23(r, acc)
            }
            5 => {
                acc.push('5');
                s23(r, acc)
            }
            6 => {
                acc.push('6');
                s23(r, acc)
            }
            7 => {
                acc.push('7');
                s23(r, acc)
            }
            8 => {
                acc.push('8');
                s23(r, acc)
            }
            9 => {
                acc.push('9');
                s23(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '0' -> '0' -> '0'
    pub fn s25<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..10) {
            0 => {
                acc.push('0');
                s3(r, acc)
            }
            1 => {
                acc.push('1');
                s3(r, acc)
            }
            2 => {
                acc.push('2');
                s3(r, acc)
            }
            3 => {
                acc.push('3');
                s3(r, acc)
            }
            4 => {
                acc.push('4');
                s3(r, acc)
            }
            5 => {
                acc.push('5');
                s3(r, acc)
            }
            6 => {
                acc.push('6');
                s3(r, acc)
            }
            7 => {
                acc.push('7');
                s3(r, acc)
            }
            8 => {
                acc.push('8');
                s3(r, acc)
            }
            9 => {
                acc.push('9');
                s3(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '0' -> '0' -> '0' -> '0'
    pub fn s26<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..10) {
            0 => {
                acc.push('0');
                s25(r, acc)
            }
            1 => {
                acc.push('1');
                s25(r, acc)
            }
            2 => {
                acc.push('2');
                s25(r, acc)
            }
            3 => {
                acc.push('3');
                s25(r, acc)
            }
            4 => {
                acc.push('4');
                s25(r, acc)
            }
            5 => {
                acc.push('5');
                s25(r, acc)
            }
            6 => {
                acc.push('6');
                s25(r, acc)
            }
            7 => {
                acc.push('7');
                s25(r, acc)
            }
            8 => {
                acc.push('8');
                s25(r, acc)
            }
            9 => {
                acc.push('9');
                s25(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
}
