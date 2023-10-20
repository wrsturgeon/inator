#[inline(always)]
pub fn abc_tuple<I: Iterator<Item = char>>(
    i: I,
) -> Option<crate::inator_config::abc_tuple::Output> {
    abc_tuple_states::s0(i, crate::inator_config::abc_tuple::initial())
}
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[allow(unused_parens)]
#[allow(unused_variables)]
pub(crate) mod abc_tuple_states {
    #[inline]
    ///Minimal input to reach this state: [this is the initial state]
    pub fn s0<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::abc_tuple::Output,
    ) -> Option<crate::inator_config::abc_tuple::Output> {
        match i.next() {
            Some(token @ ('(')) => s2(i, acc),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '(' -> 'A' -> ',' -> 'A' -> ',' -> 'A' -> '\n'
    pub fn s1<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::abc_tuple::Output,
    ) -> Option<crate::inator_config::abc_tuple::Output> {
        match i.next() {
            Some(token @ ('\n' | ' ')) => s1(i, acc),
            Some(token @ ('\r')) => s6(i, acc),
            Some(token @ (')')) => s10(i, acc),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '('
    pub fn s2<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::abc_tuple::Output,
    ) -> Option<crate::inator_config::abc_tuple::Output> {
        match i.next() {
            Some(token @ ('\n' | ' ')) => s2(i, acc),
            Some(token @ ('\r')) => s7(i, acc),
            Some(token @ (')')) => s10(i, acc),
            Some(token @ ('A' | 'B' | 'C')) => {
                s11(i, crate::inator_config::abc_tuple::append(acc, token))
            }
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '(' -> 'A' -> ','
    pub fn s3<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::abc_tuple::Output,
    ) -> Option<crate::inator_config::abc_tuple::Output> {
        match i.next() {
            Some(token @ ('\n' | ' ')) => s3(i, acc),
            Some(token @ ('A' | 'B' | 'C')) => {
                s5(i, crate::inator_config::abc_tuple::append(acc, token))
            }
            Some(token @ ('\r')) => s8(i, acc),
            Some(token @ (')')) => s10(i, acc),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '(' -> 'A' -> ',' -> 'A' -> ',' -> 'A'
    pub fn s4<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::abc_tuple::Output,
    ) -> Option<crate::inator_config::abc_tuple::Output> {
        match i.next() {
            Some(token @ ('\n' | ' ')) => s1(i, acc),
            Some(token @ ('\r')) => s6(i, acc),
            Some(token @ (')')) => s10(i, acc),
            Some(token @ (',')) => s13(i, acc),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '(' -> 'A' -> ',' -> 'A'
    pub fn s5<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::abc_tuple::Output,
    ) -> Option<crate::inator_config::abc_tuple::Output> {
        match i.next() {
            Some(token @ ('\n' | ' ')) => s5(i, acc),
            Some(token @ ('\r')) => s9(i, acc),
            Some(token @ (')')) => s10(i, acc),
            Some(token @ (',')) => s13(i, acc),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '(' -> 'A' -> ',' -> 'A' -> ',' -> 'A' -> '\r'
    pub fn s6<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::abc_tuple::Output,
    ) -> Option<crate::inator_config::abc_tuple::Output> {
        match i.next() {
            Some(token @ ('\n')) => s1(i, acc),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '(' -> '\r'
    pub fn s7<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::abc_tuple::Output,
    ) -> Option<crate::inator_config::abc_tuple::Output> {
        match i.next() {
            Some(token @ ('\n')) => s2(i, acc),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '(' -> 'A' -> ',' -> '\r'
    pub fn s8<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::abc_tuple::Output,
    ) -> Option<crate::inator_config::abc_tuple::Output> {
        match i.next() {
            Some(token @ ('\n')) => s3(i, acc),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '(' -> 'A' -> ',' -> 'A' -> '\r'
    pub fn s9<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::abc_tuple::Output,
    ) -> Option<crate::inator_config::abc_tuple::Output> {
        match i.next() {
            Some(token @ ('\n')) => s5(i, acc),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '(' -> ')'
    pub fn s10<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::abc_tuple::Output,
    ) -> Option<crate::inator_config::abc_tuple::Output> {
        match i.next() {
            None => Some(acc),
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '(' -> 'A'
    pub fn s11<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::abc_tuple::Output,
    ) -> Option<crate::inator_config::abc_tuple::Output> {
        match i.next() {
            Some(token @ (',')) => s3(i, acc),
            Some(token @ ('\n' | ' ')) => s11(i, acc),
            Some(token @ ('\r')) => s12(i, acc),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '(' -> 'A' -> '\r'
    pub fn s12<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::abc_tuple::Output,
    ) -> Option<crate::inator_config::abc_tuple::Output> {
        match i.next() {
            Some(token @ ('\n')) => s11(i, acc),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '(' -> 'A' -> ',' -> 'A' -> ','
    pub fn s13<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::abc_tuple::Output,
    ) -> Option<crate::inator_config::abc_tuple::Output> {
        match i.next() {
            Some(token @ ('A' | 'B' | 'C')) => {
                s4(i, crate::inator_config::abc_tuple::append(acc, token))
            }
            Some(token @ ('\n' | ' ')) => s13(i, acc),
            Some(token @ ('\r')) => s14(i, acc),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '(' -> 'A' -> ',' -> 'A' -> ',' -> '\r'
    pub fn s14<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::abc_tuple::Output,
    ) -> Option<crate::inator_config::abc_tuple::Output> {
        match i.next() {
            Some(token @ ('\n')) => s13(i, acc),
            None => None,
            _ => None,
        }
    }
}
#[inline]
pub fn abc_tuple_fuzz<R: rand::Rng>(r: &mut R) -> Vec<char> {
    let mut v = abc_tuple_fuzz_states::s14(r, vec![]);
    v.reverse();
    v
}
#[allow(non_snake_case)]
#[allow(unused_mut)]
mod abc_tuple_fuzz_states {
    #[inline]
    ///Minimal input to reach this state: ')' -> '('
    pub fn s0<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        if r.gen() {
            return acc;
        }
        s14(r, vec![])
    }
    #[inline]
    ///Minimal input to reach this state: ')'
    pub fn s1<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..7) {
            0 => {
                acc.push('\n');
                s2(r, acc)
            }
            1 => {
                acc.push(' ');
                s1(r, acc)
            }
            2 => {
                acc.push('(');
                s0(r, acc)
            }
            3 => {
                acc.push(',');
                s15(r, acc)
            }
            4 => {
                acc.push('A');
                s10(r, acc)
            }
            5 => {
                acc.push('B');
                s10(r, acc)
            }
            6 => {
                acc.push('C');
                s10(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: ')' -> '\n'
    pub fn s2<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..8) {
            0 => {
                acc.push('\n');
                s2(r, acc)
            }
            1 => {
                acc.push('\r');
                s1(r, acc)
            }
            2 => {
                acc.push(' ');
                s1(r, acc)
            }
            3 => {
                acc.push('(');
                s0(r, acc)
            }
            4 => {
                acc.push(',');
                s15(r, acc)
            }
            5 => {
                acc.push('A');
                s10(r, acc)
            }
            6 => {
                acc.push('B');
                s10(r, acc)
            }
            7 => {
                acc.push('C');
                s10(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: ')' -> ',' -> 'A'
    pub fn s3<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..3) {
            0 => {
                acc.push('\n');
                s8(r, acc)
            }
            1 => {
                acc.push(' ');
                s3(r, acc)
            }
            2 => {
                acc.push('(');
                s0(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: ')' -> 'A' -> ',' -> '\n' -> 'A'
    pub fn s4<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..4) {
            0 => {
                acc.push('\n');
                s5(r, acc)
            }
            1 => {
                acc.push(' ');
                s4(r, acc)
            }
            2 => {
                acc.push('(');
                s0(r, acc)
            }
            3 => {
                acc.push(',');
                s15(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: ')' -> 'A' -> ',' -> '\n' -> 'A' -> '\n'
    pub fn s5<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..5) {
            0 => {
                acc.push('\n');
                s5(r, acc)
            }
            1 => {
                acc.push('\r');
                s4(r, acc)
            }
            2 => {
                acc.push(' ');
                s4(r, acc)
            }
            3 => {
                acc.push('(');
                s0(r, acc)
            }
            4 => {
                acc.push(',');
                s15(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: ')' -> 'A' -> ',' -> 'A' -> '\n'
    pub fn s6<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..5) {
            0 => {
                acc.push('\n');
                s6(r, acc)
            }
            1 => {
                acc.push('\r');
                s7(r, acc)
            }
            2 => {
                acc.push(' ');
                s7(r, acc)
            }
            3 => {
                acc.push('(');
                s0(r, acc)
            }
            4 => {
                acc.push(',');
                s11(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: ')' -> 'A' -> ',' -> 'A'
    pub fn s7<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..4) {
            0 => {
                acc.push('\n');
                s6(r, acc)
            }
            1 => {
                acc.push(' ');
                s7(r, acc)
            }
            2 => {
                acc.push('(');
                s0(r, acc)
            }
            3 => {
                acc.push(',');
                s11(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: ')' -> ',' -> 'A' -> '\n'
    pub fn s8<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..4) {
            0 => {
                acc.push('\n');
                s8(r, acc)
            }
            1 => {
                acc.push('\r');
                s3(r, acc)
            }
            2 => {
                acc.push(' ');
                s3(r, acc)
            }
            3 => {
                acc.push('(');
                s0(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: ')' -> 'A' -> '\n'
    pub fn s9<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..4) {
            0 => {
                acc.push('\n');
                s9(r, acc)
            }
            1 => {
                acc.push('\r');
                s10(r, acc)
            }
            2 => {
                acc.push(' ');
                s10(r, acc)
            }
            3 => {
                acc.push(',');
                s11(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: ')' -> 'A'
    pub fn s10<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..3) {
            0 => {
                acc.push('\n');
                s9(r, acc)
            }
            1 => {
                acc.push(' ');
                s10(r, acc)
            }
            2 => {
                acc.push(',');
                s11(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: ')' -> 'A' -> ','
    pub fn s11<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..5) {
            0 => {
                acc.push('\n');
                s12(r, acc)
            }
            1 => {
                acc.push(' ');
                s13(r, acc)
            }
            2 => {
                acc.push('A');
                s7(r, acc)
            }
            3 => {
                acc.push('B');
                s7(r, acc)
            }
            4 => {
                acc.push('C');
                s7(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: ')' -> 'A' -> ',' -> '\n'
    pub fn s12<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..6) {
            0 => {
                acc.push('\n');
                s12(r, acc)
            }
            1 => {
                acc.push('\r');
                s13(r, acc)
            }
            2 => {
                acc.push(' ');
                s13(r, acc)
            }
            3 => {
                acc.push('A');
                s4(r, acc)
            }
            4 => {
                acc.push('B');
                s4(r, acc)
            }
            5 => {
                acc.push('C');
                s4(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: ')' -> 'A' -> ',' -> ' '
    pub fn s13<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..5) {
            0 => {
                acc.push('\n');
                s12(r, acc)
            }
            1 => {
                acc.push(' ');
                s13(r, acc)
            }
            2 => {
                acc.push('A');
                s4(r, acc)
            }
            3 => {
                acc.push('B');
                s4(r, acc)
            }
            4 => {
                acc.push('C');
                s4(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: [this is the initial state]
    pub fn s14<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        acc.push(')');
        s1(r, acc)
    }
    #[inline]
    ///Minimal input to reach this state: ')' -> ','
    pub fn s15<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..5) {
            0 => {
                acc.push('\n');
                s16(r, acc)
            }
            1 => {
                acc.push(' ');
                s15(r, acc)
            }
            2 => {
                acc.push('A');
                s3(r, acc)
            }
            3 => {
                acc.push('B');
                s3(r, acc)
            }
            4 => {
                acc.push('C');
                s3(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: ')' -> ',' -> '\n'
    pub fn s16<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..6) {
            0 => {
                acc.push('\n');
                s16(r, acc)
            }
            1 => {
                acc.push('\r');
                s15(r, acc)
            }
            2 => {
                acc.push(' ');
                s15(r, acc)
            }
            3 => {
                acc.push('A');
                s3(r, acc)
            }
            4 => {
                acc.push('B');
                s3(r, acc)
            }
            5 => {
                acc.push('C');
                s3(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
}
