#[inline(always)]
pub fn json<I: Iterator<Item = char>>(
    i: I,
) -> Option<crate::inator_config::json::Output> {
    json_states::s0(i, crate::inator_config::json::initial())
}
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[allow(unused_parens)]
#[allow(unused_variables)]
pub(crate) mod json_states {
    #[inline]
    ///Minimal input to reach this state: [this is the initial state]
    pub fn s0<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::json::Output,
    ) -> Option<crate::inator_config::json::Output> {
        match i.next() {
            Some(token @ ('\n' | ' ')) => s0(i, acc),
            Some(token @ ('\r')) => s1(i, acc),
            Some(token @ ('-')) => s2(i, crate::inator_config::json::negative(acc)),
            Some(token @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                s3(i, crate::inator_config::json::digit(acc))
            }
            Some(token @ ('"')) => s11(i, crate::inator_config::json::begin_string(acc)),
            Some(token @ ('}')) => s13(i, crate::inator_config::json::close_object(acc)),
            Some(token @ (']')) => s13(i, crate::inator_config::json::end_array(acc)),
            Some(token @ ('[')) => s13(i, crate::inator_config::json::open_array(acc)),
            Some(token @ ('t')) => s15(i, acc),
            Some(token @ ('n')) => s18(i, acc),
            Some(token @ ('{')) => s21(i, crate::inator_config::json::open_object(acc)),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '\r'
    pub fn s1<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::json::Output,
    ) -> Option<crate::inator_config::json::Output> {
        match i.next() {
            Some(token @ ('\n')) => s0(i, acc),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '-'
    pub fn s2<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::json::Output,
    ) -> Option<crate::inator_config::json::Output> {
        match i.next() {
            Some(token @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                s3(i, crate::inator_config::json::digit(acc))
            }
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0'
    pub fn s3<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::json::Output,
    ) -> Option<crate::inator_config::json::Output> {
        match i.next() {
            Some(token @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                s3(i, crate::inator_config::json::digit(acc))
            }
            Some(token @ ('.')) => s4(i, crate::inator_config::json::fraction_dot(acc)),
            Some(token @ ('E' | 'e')) => {
                s6(i, crate::inator_config::json::exponential(acc))
            }
            Some(token @ ('\n' | ' ')) => s13(i, acc),
            Some(token @ ('\r')) => s14(i, acc),
            None => Some(acc),
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '.'
    pub fn s4<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::json::Output,
    ) -> Option<crate::inator_config::json::Output> {
        match i.next() {
            Some(token @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                s5(i, crate::inator_config::json::digit(acc))
            }
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '.' -> '0'
    pub fn s5<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::json::Output,
    ) -> Option<crate::inator_config::json::Output> {
        match i.next() {
            Some(token @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                s5(i, crate::inator_config::json::digit(acc))
            }
            Some(token @ ('E' | 'e')) => {
                s6(i, crate::inator_config::json::exponential(acc))
            }
            Some(token @ ('\n' | ' ')) => s13(i, acc),
            Some(token @ ('\r')) => s14(i, acc),
            None => Some(acc),
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> 'E'
    pub fn s6<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::json::Output,
    ) -> Option<crate::inator_config::json::Output> {
        match i.next() {
            Some(token @ ('\n' | ' ')) => s6(i, acc),
            Some(token @ ('+')) => s7(i, acc),
            Some(token @ ('-')) => s7(i, crate::inator_config::json::negative(acc)),
            Some(token @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                s8(i, crate::inator_config::json::digit(acc))
            }
            Some(token @ ('\r')) => s10(i, acc),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> 'E' -> '+'
    pub fn s7<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::json::Output,
    ) -> Option<crate::inator_config::json::Output> {
        match i.next() {
            Some(token @ ('\n' | ' ')) => s7(i, acc),
            Some(token @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                s8(i, crate::inator_config::json::digit(acc))
            }
            Some(token @ ('\r')) => s9(i, acc),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> 'E' -> '0'
    pub fn s8<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::json::Output,
    ) -> Option<crate::inator_config::json::Output> {
        match i.next() {
            Some(token @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                s8(i, crate::inator_config::json::digit(acc))
            }
            Some(token @ ('\n' | ' ')) => s13(i, acc),
            Some(token @ ('\r')) => s14(i, acc),
            None => Some(acc),
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> 'E' -> '+' -> '\r'
    pub fn s9<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::json::Output,
    ) -> Option<crate::inator_config::json::Output> {
        match i.next() {
            Some(token @ ('\n')) => s7(i, acc),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> 'E' -> '\r'
    pub fn s10<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::json::Output,
    ) -> Option<crate::inator_config::json::Output> {
        match i.next() {
            Some(token @ ('\n')) => s6(i, acc),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '"'
    pub fn s11<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::json::Output,
    ) -> Option<crate::inator_config::json::Output> {
        match i.next() {
            Some(
                token @ (' ' | '!' | '#' | '$' | '%' | '&' | '\'' | '(' | ')' | '*' | '+'
                | ',' | '-' | '.' | '/' | '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7'
                | '8' | '9' | ':' | ';' | '<' | '=' | '>' | '?' | '@' | 'A' | 'B' | 'C'
                | 'D' | 'E' | 'F' | 'G' | 'H' | 'I' | 'J' | 'K' | 'L' | 'M' | 'N' | 'O'
                | 'P' | 'Q' | 'R' | 'S' | 'T' | 'U' | 'V' | 'W' | 'X' | 'Y' | 'Z' | '['
                | ']' | '^' | '_' | '`' | 'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'g' | 'h'
                | 'i' | 'j' | 'k' | 'l' | 'm' | 'n' | 'o' | 'p' | 'q' | 'r' | 's' | 't'
                | 'u' | 'v' | 'w' | 'x' | 'y' | 'z' | '{' | '|' | '}' | '~' | '\u{80}'
                | '\u{81}' | '\u{82}' | '\u{83}' | '\u{84}' | '\u{85}' | '\u{86}'
                | '\u{87}' | '\u{88}' | '\u{89}' | '\u{8a}' | '\u{8b}' | '\u{8c}'
                | '\u{8d}' | '\u{8e}' | '\u{8f}' | '\u{90}' | '\u{91}' | '\u{92}'
                | '\u{93}' | '\u{94}' | '\u{95}' | '\u{96}' | '\u{97}' | '\u{98}'
                | '\u{99}' | '\u{9a}' | '\u{9b}' | '\u{9c}' | '\u{9d}' | '\u{9e}'
                | '\u{9f}' | '\u{a0}' | '¡' | '¢' | '£' | '¤' | '¥' | '¦' | '§'
                | '¨' | '©' | 'ª' | '«' | '¬' | '\u{ad}' | '®' | '¯' | '°' | '±'
                | '²' | '³' | '´' | 'µ' | '¶' | '·' | '¸' | '¹' | 'º' | '»'
                | '¼' | '½' | '¾' | '¿' | 'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å'
                | 'Æ' | 'Ç' | 'È' | 'É' | 'Ê' | 'Ë' | 'Ì' | 'Í' | 'Î' | 'Ï'
                | 'Ð' | 'Ñ' | 'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | '×' | 'Ø' | 'Ù'
                | 'Ú' | 'Û' | 'Ü' | 'Ý' | 'Þ' | 'ß' | 'à' | 'á' | 'â' | 'ã'
                | 'ä' | 'å' | 'æ' | 'ç' | 'è' | 'é' | 'ê' | 'ë' | 'ì' | 'í'
                | 'î' | 'ï' | 'ð' | 'ñ' | 'ò' | 'ó' | 'ô' | 'õ' | 'ö' | '÷'
                | 'ø' | 'ù' | 'ú' | 'û' | 'ü' | 'ý' | 'þ'),
            ) => s11(i, crate::inator_config::json::character(acc)),
            Some(token @ ('\\')) => s12(i, acc),
            Some(token @ ('"')) => s13(i, crate::inator_config::json::end_string(acc)),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '"' -> '\\'
    pub fn s12<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::json::Output,
    ) -> Option<crate::inator_config::json::Output> {
        match i.next() {
            Some(token @ ('\\')) => {
                s11(i, crate::inator_config::json::esc_backslash(acc))
            }
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '['
    pub fn s13<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::json::Output,
    ) -> Option<crate::inator_config::json::Output> {
        match i.next() {
            Some(token @ ('\n' | ' ')) => s13(i, acc),
            Some(token @ ('\r')) => s14(i, acc),
            None => Some(acc),
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '[' -> '\r'
    pub fn s14<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::json::Output,
    ) -> Option<crate::inator_config::json::Output> {
        match i.next() {
            Some(token @ ('\n')) => s13(i, acc),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: 't'
    pub fn s15<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::json::Output,
    ) -> Option<crate::inator_config::json::Output> {
        match i.next() {
            Some(token @ ('r')) => s16(i, acc),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: 't' -> 'r'
    pub fn s16<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::json::Output,
    ) -> Option<crate::inator_config::json::Output> {
        match i.next() {
            Some(token @ ('u')) => s17(i, acc),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: 't' -> 'r' -> 'u'
    pub fn s17<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::json::Output,
    ) -> Option<crate::inator_config::json::Output> {
        match i.next() {
            Some(token @ ('e')) => s13(i, crate::inator_config::json::lit_true(acc)),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: 'n'
    pub fn s18<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::json::Output,
    ) -> Option<crate::inator_config::json::Output> {
        match i.next() {
            Some(token @ ('u')) => s19(i, acc),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: 'n' -> 'u'
    pub fn s19<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::json::Output,
    ) -> Option<crate::inator_config::json::Output> {
        match i.next() {
            Some(token @ ('l')) => s20(i, acc),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: 'n' -> 'u' -> 'l'
    pub fn s20<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::json::Output,
    ) -> Option<crate::inator_config::json::Output> {
        match i.next() {
            Some(token @ ('l')) => s13(i, crate::inator_config::json::lit_null(acc)),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '{'
    pub fn s21<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::json::Output,
    ) -> Option<crate::inator_config::json::Output> {
        match i.next() {
            Some(token @ ('\n' | ' ')) => s21(i, acc),
            Some(token @ ('\r')) => s22(i, acc),
            Some(token @ ('"')) => s23(i, crate::inator_config::json::begin_string(acc)),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '{' -> '\r'
    pub fn s22<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::json::Output,
    ) -> Option<crate::inator_config::json::Output> {
        match i.next() {
            Some(token @ ('\n')) => s21(i, acc),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '{' -> '"'
    pub fn s23<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::json::Output,
    ) -> Option<crate::inator_config::json::Output> {
        match i.next() {
            Some(
                token @ (' ' | '!' | '#' | '$' | '%' | '&' | '\'' | '(' | ')' | '*' | '+'
                | ',' | '-' | '.' | '/' | '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7'
                | '8' | '9' | ':' | ';' | '<' | '=' | '>' | '?' | '@' | 'A' | 'B' | 'C'
                | 'D' | 'E' | 'F' | 'G' | 'H' | 'I' | 'J' | 'K' | 'L' | 'M' | 'N' | 'O'
                | 'P' | 'Q' | 'R' | 'S' | 'T' | 'U' | 'V' | 'W' | 'X' | 'Y' | 'Z' | '['
                | ']' | '^' | '_' | '`' | 'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'g' | 'h'
                | 'i' | 'j' | 'k' | 'l' | 'm' | 'n' | 'o' | 'p' | 'q' | 'r' | 's' | 't'
                | 'u' | 'v' | 'w' | 'x' | 'y' | 'z' | '{' | '|' | '}' | '~' | '\u{80}'
                | '\u{81}' | '\u{82}' | '\u{83}' | '\u{84}' | '\u{85}' | '\u{86}'
                | '\u{87}' | '\u{88}' | '\u{89}' | '\u{8a}' | '\u{8b}' | '\u{8c}'
                | '\u{8d}' | '\u{8e}' | '\u{8f}' | '\u{90}' | '\u{91}' | '\u{92}'
                | '\u{93}' | '\u{94}' | '\u{95}' | '\u{96}' | '\u{97}' | '\u{98}'
                | '\u{99}' | '\u{9a}' | '\u{9b}' | '\u{9c}' | '\u{9d}' | '\u{9e}'
                | '\u{9f}' | '\u{a0}' | '¡' | '¢' | '£' | '¤' | '¥' | '¦' | '§'
                | '¨' | '©' | 'ª' | '«' | '¬' | '\u{ad}' | '®' | '¯' | '°' | '±'
                | '²' | '³' | '´' | 'µ' | '¶' | '·' | '¸' | '¹' | 'º' | '»'
                | '¼' | '½' | '¾' | '¿' | 'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å'
                | 'Æ' | 'Ç' | 'È' | 'É' | 'Ê' | 'Ë' | 'Ì' | 'Í' | 'Î' | 'Ï'
                | 'Ð' | 'Ñ' | 'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | '×' | 'Ø' | 'Ù'
                | 'Ú' | 'Û' | 'Ü' | 'Ý' | 'Þ' | 'ß' | 'à' | 'á' | 'â' | 'ã'
                | 'ä' | 'å' | 'æ' | 'ç' | 'è' | 'é' | 'ê' | 'ë' | 'ì' | 'í'
                | 'î' | 'ï' | 'ð' | 'ñ' | 'ò' | 'ó' | 'ô' | 'õ' | 'ö' | '÷'
                | 'ø' | 'ù' | 'ú' | 'û' | 'ü' | 'ý' | 'þ'),
            ) => s23(i, crate::inator_config::json::character(acc)),
            Some(token @ ('\\')) => s24(i, acc),
            Some(token @ ('"')) => s25(i, crate::inator_config::json::end_string(acc)),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '{' -> '"' -> '\\'
    pub fn s24<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::json::Output,
    ) -> Option<crate::inator_config::json::Output> {
        match i.next() {
            Some(token @ ('\\')) => {
                s23(i, crate::inator_config::json::esc_backslash(acc))
            }
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '{' -> '"' -> '"'
    pub fn s25<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::json::Output,
    ) -> Option<crate::inator_config::json::Output> {
        match i.next() {
            Some(token @ (':')) => s13(i, crate::inator_config::json::key_value_sep(acc)),
            Some(token @ ('\n' | ' ')) => s25(i, acc),
            Some(token @ ('\r')) => s26(i, acc),
            None => None,
            _ => None,
        }
    }
    #[inline]
    ///Minimal input to reach this state: '{' -> '"' -> '"' -> '\r'
    pub fn s26<I: Iterator<Item = char>>(
        mut i: I,
        acc: crate::inator_config::json::Output,
    ) -> Option<crate::inator_config::json::Output> {
        match i.next() {
            Some(token @ ('\n')) => s25(i, acc),
            None => None,
            _ => None,
        }
    }
}
#[inline]
pub fn json_fuzz<R: rand::Rng>(r: &mut R) -> Vec<char> {
    let mut v = json_fuzz_states::s11(r, vec![]);
    v.reverse();
    v
}
#[allow(non_snake_case)]
#[allow(unused_mut)]
mod json_fuzz_states {
    #[inline]
    ///Minimal input to reach this state: '['
    pub fn s0<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        if r.gen() {
            return acc;
        }
        match r.gen_range(0..2) {
            0 => {
                acc.push('\n');
                s1(r, acc)
            }
            1 => {
                acc.push(' ');
                s0(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '[' -> '\n'
    pub fn s1<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        if r.gen() {
            return acc;
        }
        match r.gen_range(0..3) {
            0 => {
                acc.push('\n');
                s1(r, acc)
            }
            1 => {
                acc.push('\r');
                s0(r, acc)
            }
            2 => {
                acc.push(' ');
                s0(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '\n'
    pub fn s2<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        if r.gen() {
            return acc;
        }
        match r.gen_range(0..7) {
            0 => {
                acc.push('\n');
                s2(r, acc)
            }
            1 => {
                acc.push('\r');
                s8(r, acc)
            }
            2 => {
                acc.push(' ');
                s8(r, acc)
            }
            3 => {
                acc.push('+');
                s13(r, acc)
            }
            4 => {
                acc.push('-');
                s13(r, acc)
            }
            5 => {
                acc.push('E');
                s10(r, acc)
            }
            6 => {
                acc.push('e');
                s10(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '-' -> '\n'
    pub fn s3<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        if r.gen() {
            return acc;
        }
        match r.gen_range(0..5) {
            0 => {
                acc.push('\n');
                s3(r, acc)
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
                acc.push('E');
                s10(r, acc)
            }
            4 => {
                acc.push('e');
                s10(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '.' -> '0'
    pub fn s4<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        if r.gen() {
            return acc;
        }
        match r.gen_range(0..13) {
            0 => {
                acc.push('\n');
                s1(r, acc)
            }
            1 => {
                acc.push(' ');
                s0(r, acc)
            }
            2 => {
                acc.push('-');
                s0(r, acc)
            }
            3 => {
                acc.push('0');
                s4(r, acc)
            }
            4 => {
                acc.push('1');
                s4(r, acc)
            }
            5 => {
                acc.push('2');
                s4(r, acc)
            }
            6 => {
                acc.push('3');
                s4(r, acc)
            }
            7 => {
                acc.push('4');
                s4(r, acc)
            }
            8 => {
                acc.push('5');
                s4(r, acc)
            }
            9 => {
                acc.push('6');
                s4(r, acc)
            }
            10 => {
                acc.push('7');
                s4(r, acc)
            }
            11 => {
                acc.push('8');
                s4(r, acc)
            }
            12 => {
                acc.push('9');
                s4(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> 'E' -> '0'
    pub fn s5<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        if r.gen() {
            return acc;
        }
        match r.gen_range(0..14) {
            0 => {
                acc.push('\n');
                s1(r, acc)
            }
            1 => {
                acc.push(' ');
                s0(r, acc)
            }
            2 => {
                acc.push('-');
                s0(r, acc)
            }
            3 => {
                acc.push('.');
                s9(r, acc)
            }
            4 => {
                acc.push('0');
                s5(r, acc)
            }
            5 => {
                acc.push('1');
                s5(r, acc)
            }
            6 => {
                acc.push('2');
                s5(r, acc)
            }
            7 => {
                acc.push('3');
                s5(r, acc)
            }
            8 => {
                acc.push('4');
                s5(r, acc)
            }
            9 => {
                acc.push('5');
                s5(r, acc)
            }
            10 => {
                acc.push('6');
                s5(r, acc)
            }
            11 => {
                acc.push('7');
                s5(r, acc)
            }
            12 => {
                acc.push('8');
                s5(r, acc)
            }
            13 => {
                acc.push('9');
                s5(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0'
    pub fn s6<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        if r.gen() {
            return acc;
        }
        match r.gen_range(0..17) {
            0 => {
                acc.push('\n');
                s2(r, acc)
            }
            1 => {
                acc.push(' ');
                s8(r, acc)
            }
            2 => {
                acc.push('+');
                s13(r, acc)
            }
            3 => {
                acc.push('-');
                s7(r, acc)
            }
            4 => {
                acc.push('.');
                s9(r, acc)
            }
            5 => {
                acc.push('0');
                s6(r, acc)
            }
            6 => {
                acc.push('1');
                s6(r, acc)
            }
            7 => {
                acc.push('2');
                s6(r, acc)
            }
            8 => {
                acc.push('3');
                s6(r, acc)
            }
            9 => {
                acc.push('4');
                s6(r, acc)
            }
            10 => {
                acc.push('5');
                s6(r, acc)
            }
            11 => {
                acc.push('6');
                s6(r, acc)
            }
            12 => {
                acc.push('7');
                s6(r, acc)
            }
            13 => {
                acc.push('8');
                s6(r, acc)
            }
            14 => {
                acc.push('9');
                s6(r, acc)
            }
            15 => {
                acc.push('E');
                s10(r, acc)
            }
            16 => {
                acc.push('e');
                s10(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '-'
    pub fn s7<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        if r.gen() {
            return acc;
        }
        match r.gen_range(0..4) {
            0 => {
                acc.push('\n');
                s3(r, acc)
            }
            1 => {
                acc.push(' ');
                s7(r, acc)
            }
            2 => {
                acc.push('E');
                s10(r, acc)
            }
            3 => {
                acc.push('e');
                s10(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> ' '
    pub fn s8<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        if r.gen() {
            return acc;
        }
        match r.gen_range(0..6) {
            0 => {
                acc.push('\n');
                s2(r, acc)
            }
            1 => {
                acc.push(' ');
                s8(r, acc)
            }
            2 => {
                acc.push('+');
                s13(r, acc)
            }
            3 => {
                acc.push('-');
                s13(r, acc)
            }
            4 => {
                acc.push('E');
                s10(r, acc)
            }
            5 => {
                acc.push('e');
                s10(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '.'
    pub fn s9<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
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
    ///Minimal input to reach this state: '0' -> 'E'
    pub fn s10<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
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
    ///Minimal input to reach this state: [this is the initial state]
    pub fn s11<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..19) {
            0 => {
                acc.push('\n');
                s12(r, acc)
            }
            1 => {
                acc.push(' ');
                s11(r, acc)
            }
            2 => {
                acc.push('"');
                s15(r, acc)
            }
            3 => {
                acc.push('0');
                s6(r, acc)
            }
            4 => {
                acc.push('1');
                s6(r, acc)
            }
            5 => {
                acc.push('2');
                s6(r, acc)
            }
            6 => {
                acc.push('3');
                s6(r, acc)
            }
            7 => {
                acc.push('4');
                s6(r, acc)
            }
            8 => {
                acc.push('5');
                s6(r, acc)
            }
            9 => {
                acc.push('6');
                s6(r, acc)
            }
            10 => {
                acc.push('7');
                s6(r, acc)
            }
            11 => {
                acc.push('8');
                s6(r, acc)
            }
            12 => {
                acc.push('9');
                s6(r, acc)
            }
            13 => {
                acc.push(':');
                s27(r, acc)
            }
            14 => {
                acc.push('[');
                s0(r, acc)
            }
            15 => {
                acc.push(']');
                s0(r, acc)
            }
            16 => {
                acc.push('e');
                s19(r, acc)
            }
            17 => {
                acc.push('l');
                s22(r, acc)
            }
            18 => {
                acc.push('}');
                s0(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '\n'
    pub fn s12<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..20) {
            0 => {
                acc.push('\n');
                s12(r, acc)
            }
            1 => {
                acc.push('\r');
                s11(r, acc)
            }
            2 => {
                acc.push(' ');
                s11(r, acc)
            }
            3 => {
                acc.push('"');
                s15(r, acc)
            }
            4 => {
                acc.push('0');
                s6(r, acc)
            }
            5 => {
                acc.push('1');
                s6(r, acc)
            }
            6 => {
                acc.push('2');
                s6(r, acc)
            }
            7 => {
                acc.push('3');
                s6(r, acc)
            }
            8 => {
                acc.push('4');
                s6(r, acc)
            }
            9 => {
                acc.push('5');
                s6(r, acc)
            }
            10 => {
                acc.push('6');
                s6(r, acc)
            }
            11 => {
                acc.push('7');
                s6(r, acc)
            }
            12 => {
                acc.push('8');
                s6(r, acc)
            }
            13 => {
                acc.push('9');
                s6(r, acc)
            }
            14 => {
                acc.push(':');
                s27(r, acc)
            }
            15 => {
                acc.push('[');
                s0(r, acc)
            }
            16 => {
                acc.push(']');
                s0(r, acc)
            }
            17 => {
                acc.push('e');
                s19(r, acc)
            }
            18 => {
                acc.push('l');
                s22(r, acc)
            }
            19 => {
                acc.push('}');
                s0(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '+'
    pub fn s13<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..4) {
            0 => {
                acc.push('\n');
                s14(r, acc)
            }
            1 => {
                acc.push(' ');
                s13(r, acc)
            }
            2 => {
                acc.push('E');
                s10(r, acc)
            }
            3 => {
                acc.push('e');
                s10(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '0' -> '+' -> '\n'
    pub fn s14<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..5) {
            0 => {
                acc.push('\n');
                s14(r, acc)
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
                acc.push('E');
                s10(r, acc)
            }
            4 => {
                acc.push('e');
                s10(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '"'
    pub fn s15<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..222) {
            0 => {
                acc.push(' ');
                s15(r, acc)
            }
            1 => {
                acc.push('!');
                s15(r, acc)
            }
            2 => {
                acc.push('"');
                s0(r, acc)
            }
            3 => {
                acc.push('#');
                s15(r, acc)
            }
            4 => {
                acc.push('$');
                s15(r, acc)
            }
            5 => {
                acc.push('%');
                s15(r, acc)
            }
            6 => {
                acc.push('&');
                s15(r, acc)
            }
            7 => {
                acc.push('\'');
                s15(r, acc)
            }
            8 => {
                acc.push('(');
                s15(r, acc)
            }
            9 => {
                acc.push(')');
                s15(r, acc)
            }
            10 => {
                acc.push('*');
                s15(r, acc)
            }
            11 => {
                acc.push('+');
                s15(r, acc)
            }
            12 => {
                acc.push(',');
                s15(r, acc)
            }
            13 => {
                acc.push('-');
                s15(r, acc)
            }
            14 => {
                acc.push('.');
                s15(r, acc)
            }
            15 => {
                acc.push('/');
                s15(r, acc)
            }
            16 => {
                acc.push('0');
                s15(r, acc)
            }
            17 => {
                acc.push('1');
                s15(r, acc)
            }
            18 => {
                acc.push('2');
                s15(r, acc)
            }
            19 => {
                acc.push('3');
                s15(r, acc)
            }
            20 => {
                acc.push('4');
                s15(r, acc)
            }
            21 => {
                acc.push('5');
                s15(r, acc)
            }
            22 => {
                acc.push('6');
                s15(r, acc)
            }
            23 => {
                acc.push('7');
                s15(r, acc)
            }
            24 => {
                acc.push('8');
                s15(r, acc)
            }
            25 => {
                acc.push('9');
                s15(r, acc)
            }
            26 => {
                acc.push(':');
                s15(r, acc)
            }
            27 => {
                acc.push(';');
                s15(r, acc)
            }
            28 => {
                acc.push('<');
                s15(r, acc)
            }
            29 => {
                acc.push('=');
                s15(r, acc)
            }
            30 => {
                acc.push('>');
                s15(r, acc)
            }
            31 => {
                acc.push('?');
                s15(r, acc)
            }
            32 => {
                acc.push('@');
                s15(r, acc)
            }
            33 => {
                acc.push('A');
                s15(r, acc)
            }
            34 => {
                acc.push('B');
                s15(r, acc)
            }
            35 => {
                acc.push('C');
                s15(r, acc)
            }
            36 => {
                acc.push('D');
                s15(r, acc)
            }
            37 => {
                acc.push('E');
                s15(r, acc)
            }
            38 => {
                acc.push('F');
                s15(r, acc)
            }
            39 => {
                acc.push('G');
                s15(r, acc)
            }
            40 => {
                acc.push('H');
                s15(r, acc)
            }
            41 => {
                acc.push('I');
                s15(r, acc)
            }
            42 => {
                acc.push('J');
                s15(r, acc)
            }
            43 => {
                acc.push('K');
                s15(r, acc)
            }
            44 => {
                acc.push('L');
                s15(r, acc)
            }
            45 => {
                acc.push('M');
                s15(r, acc)
            }
            46 => {
                acc.push('N');
                s15(r, acc)
            }
            47 => {
                acc.push('O');
                s15(r, acc)
            }
            48 => {
                acc.push('P');
                s15(r, acc)
            }
            49 => {
                acc.push('Q');
                s15(r, acc)
            }
            50 => {
                acc.push('R');
                s15(r, acc)
            }
            51 => {
                acc.push('S');
                s15(r, acc)
            }
            52 => {
                acc.push('T');
                s15(r, acc)
            }
            53 => {
                acc.push('U');
                s15(r, acc)
            }
            54 => {
                acc.push('V');
                s15(r, acc)
            }
            55 => {
                acc.push('W');
                s15(r, acc)
            }
            56 => {
                acc.push('X');
                s15(r, acc)
            }
            57 => {
                acc.push('Y');
                s15(r, acc)
            }
            58 => {
                acc.push('Z');
                s15(r, acc)
            }
            59 => {
                acc.push('[');
                s15(r, acc)
            }
            60 => {
                acc.push('\\');
                s16(r, acc)
            }
            61 => {
                acc.push(']');
                s15(r, acc)
            }
            62 => {
                acc.push('^');
                s15(r, acc)
            }
            63 => {
                acc.push('_');
                s15(r, acc)
            }
            64 => {
                acc.push('`');
                s15(r, acc)
            }
            65 => {
                acc.push('a');
                s15(r, acc)
            }
            66 => {
                acc.push('b');
                s15(r, acc)
            }
            67 => {
                acc.push('c');
                s15(r, acc)
            }
            68 => {
                acc.push('d');
                s15(r, acc)
            }
            69 => {
                acc.push('e');
                s15(r, acc)
            }
            70 => {
                acc.push('f');
                s15(r, acc)
            }
            71 => {
                acc.push('g');
                s15(r, acc)
            }
            72 => {
                acc.push('h');
                s15(r, acc)
            }
            73 => {
                acc.push('i');
                s15(r, acc)
            }
            74 => {
                acc.push('j');
                s15(r, acc)
            }
            75 => {
                acc.push('k');
                s15(r, acc)
            }
            76 => {
                acc.push('l');
                s15(r, acc)
            }
            77 => {
                acc.push('m');
                s15(r, acc)
            }
            78 => {
                acc.push('n');
                s15(r, acc)
            }
            79 => {
                acc.push('o');
                s15(r, acc)
            }
            80 => {
                acc.push('p');
                s15(r, acc)
            }
            81 => {
                acc.push('q');
                s15(r, acc)
            }
            82 => {
                acc.push('r');
                s15(r, acc)
            }
            83 => {
                acc.push('s');
                s15(r, acc)
            }
            84 => {
                acc.push('t');
                s15(r, acc)
            }
            85 => {
                acc.push('u');
                s15(r, acc)
            }
            86 => {
                acc.push('v');
                s15(r, acc)
            }
            87 => {
                acc.push('w');
                s15(r, acc)
            }
            88 => {
                acc.push('x');
                s15(r, acc)
            }
            89 => {
                acc.push('y');
                s15(r, acc)
            }
            90 => {
                acc.push('z');
                s15(r, acc)
            }
            91 => {
                acc.push('{');
                s15(r, acc)
            }
            92 => {
                acc.push('|');
                s15(r, acc)
            }
            93 => {
                acc.push('}');
                s15(r, acc)
            }
            94 => {
                acc.push('~');
                s15(r, acc)
            }
            95 => {
                acc.push('\u{80}');
                s15(r, acc)
            }
            96 => {
                acc.push('\u{81}');
                s15(r, acc)
            }
            97 => {
                acc.push('\u{82}');
                s15(r, acc)
            }
            98 => {
                acc.push('\u{83}');
                s15(r, acc)
            }
            99 => {
                acc.push('\u{84}');
                s15(r, acc)
            }
            100 => {
                acc.push('\u{85}');
                s15(r, acc)
            }
            101 => {
                acc.push('\u{86}');
                s15(r, acc)
            }
            102 => {
                acc.push('\u{87}');
                s15(r, acc)
            }
            103 => {
                acc.push('\u{88}');
                s15(r, acc)
            }
            104 => {
                acc.push('\u{89}');
                s15(r, acc)
            }
            105 => {
                acc.push('\u{8a}');
                s15(r, acc)
            }
            106 => {
                acc.push('\u{8b}');
                s15(r, acc)
            }
            107 => {
                acc.push('\u{8c}');
                s15(r, acc)
            }
            108 => {
                acc.push('\u{8d}');
                s15(r, acc)
            }
            109 => {
                acc.push('\u{8e}');
                s15(r, acc)
            }
            110 => {
                acc.push('\u{8f}');
                s15(r, acc)
            }
            111 => {
                acc.push('\u{90}');
                s15(r, acc)
            }
            112 => {
                acc.push('\u{91}');
                s15(r, acc)
            }
            113 => {
                acc.push('\u{92}');
                s15(r, acc)
            }
            114 => {
                acc.push('\u{93}');
                s15(r, acc)
            }
            115 => {
                acc.push('\u{94}');
                s15(r, acc)
            }
            116 => {
                acc.push('\u{95}');
                s15(r, acc)
            }
            117 => {
                acc.push('\u{96}');
                s15(r, acc)
            }
            118 => {
                acc.push('\u{97}');
                s15(r, acc)
            }
            119 => {
                acc.push('\u{98}');
                s15(r, acc)
            }
            120 => {
                acc.push('\u{99}');
                s15(r, acc)
            }
            121 => {
                acc.push('\u{9a}');
                s15(r, acc)
            }
            122 => {
                acc.push('\u{9b}');
                s15(r, acc)
            }
            123 => {
                acc.push('\u{9c}');
                s15(r, acc)
            }
            124 => {
                acc.push('\u{9d}');
                s15(r, acc)
            }
            125 => {
                acc.push('\u{9e}');
                s15(r, acc)
            }
            126 => {
                acc.push('\u{9f}');
                s15(r, acc)
            }
            127 => {
                acc.push('\u{a0}');
                s15(r, acc)
            }
            128 => {
                acc.push('¡');
                s15(r, acc)
            }
            129 => {
                acc.push('¢');
                s15(r, acc)
            }
            130 => {
                acc.push('£');
                s15(r, acc)
            }
            131 => {
                acc.push('¤');
                s15(r, acc)
            }
            132 => {
                acc.push('¥');
                s15(r, acc)
            }
            133 => {
                acc.push('¦');
                s15(r, acc)
            }
            134 => {
                acc.push('§');
                s15(r, acc)
            }
            135 => {
                acc.push('¨');
                s15(r, acc)
            }
            136 => {
                acc.push('©');
                s15(r, acc)
            }
            137 => {
                acc.push('ª');
                s15(r, acc)
            }
            138 => {
                acc.push('«');
                s15(r, acc)
            }
            139 => {
                acc.push('¬');
                s15(r, acc)
            }
            140 => {
                acc.push('\u{ad}');
                s15(r, acc)
            }
            141 => {
                acc.push('®');
                s15(r, acc)
            }
            142 => {
                acc.push('¯');
                s15(r, acc)
            }
            143 => {
                acc.push('°');
                s15(r, acc)
            }
            144 => {
                acc.push('±');
                s15(r, acc)
            }
            145 => {
                acc.push('²');
                s15(r, acc)
            }
            146 => {
                acc.push('³');
                s15(r, acc)
            }
            147 => {
                acc.push('´');
                s15(r, acc)
            }
            148 => {
                acc.push('µ');
                s15(r, acc)
            }
            149 => {
                acc.push('¶');
                s15(r, acc)
            }
            150 => {
                acc.push('·');
                s15(r, acc)
            }
            151 => {
                acc.push('¸');
                s15(r, acc)
            }
            152 => {
                acc.push('¹');
                s15(r, acc)
            }
            153 => {
                acc.push('º');
                s15(r, acc)
            }
            154 => {
                acc.push('»');
                s15(r, acc)
            }
            155 => {
                acc.push('¼');
                s15(r, acc)
            }
            156 => {
                acc.push('½');
                s15(r, acc)
            }
            157 => {
                acc.push('¾');
                s15(r, acc)
            }
            158 => {
                acc.push('¿');
                s15(r, acc)
            }
            159 => {
                acc.push('À');
                s15(r, acc)
            }
            160 => {
                acc.push('Á');
                s15(r, acc)
            }
            161 => {
                acc.push('Â');
                s15(r, acc)
            }
            162 => {
                acc.push('Ã');
                s15(r, acc)
            }
            163 => {
                acc.push('Ä');
                s15(r, acc)
            }
            164 => {
                acc.push('Å');
                s15(r, acc)
            }
            165 => {
                acc.push('Æ');
                s15(r, acc)
            }
            166 => {
                acc.push('Ç');
                s15(r, acc)
            }
            167 => {
                acc.push('È');
                s15(r, acc)
            }
            168 => {
                acc.push('É');
                s15(r, acc)
            }
            169 => {
                acc.push('Ê');
                s15(r, acc)
            }
            170 => {
                acc.push('Ë');
                s15(r, acc)
            }
            171 => {
                acc.push('Ì');
                s15(r, acc)
            }
            172 => {
                acc.push('Í');
                s15(r, acc)
            }
            173 => {
                acc.push('Î');
                s15(r, acc)
            }
            174 => {
                acc.push('Ï');
                s15(r, acc)
            }
            175 => {
                acc.push('Ð');
                s15(r, acc)
            }
            176 => {
                acc.push('Ñ');
                s15(r, acc)
            }
            177 => {
                acc.push('Ò');
                s15(r, acc)
            }
            178 => {
                acc.push('Ó');
                s15(r, acc)
            }
            179 => {
                acc.push('Ô');
                s15(r, acc)
            }
            180 => {
                acc.push('Õ');
                s15(r, acc)
            }
            181 => {
                acc.push('Ö');
                s15(r, acc)
            }
            182 => {
                acc.push('×');
                s15(r, acc)
            }
            183 => {
                acc.push('Ø');
                s15(r, acc)
            }
            184 => {
                acc.push('Ù');
                s15(r, acc)
            }
            185 => {
                acc.push('Ú');
                s15(r, acc)
            }
            186 => {
                acc.push('Û');
                s15(r, acc)
            }
            187 => {
                acc.push('Ü');
                s15(r, acc)
            }
            188 => {
                acc.push('Ý');
                s15(r, acc)
            }
            189 => {
                acc.push('Þ');
                s15(r, acc)
            }
            190 => {
                acc.push('ß');
                s15(r, acc)
            }
            191 => {
                acc.push('à');
                s15(r, acc)
            }
            192 => {
                acc.push('á');
                s15(r, acc)
            }
            193 => {
                acc.push('â');
                s15(r, acc)
            }
            194 => {
                acc.push('ã');
                s15(r, acc)
            }
            195 => {
                acc.push('ä');
                s15(r, acc)
            }
            196 => {
                acc.push('å');
                s15(r, acc)
            }
            197 => {
                acc.push('æ');
                s15(r, acc)
            }
            198 => {
                acc.push('ç');
                s15(r, acc)
            }
            199 => {
                acc.push('è');
                s15(r, acc)
            }
            200 => {
                acc.push('é');
                s15(r, acc)
            }
            201 => {
                acc.push('ê');
                s15(r, acc)
            }
            202 => {
                acc.push('ë');
                s15(r, acc)
            }
            203 => {
                acc.push('ì');
                s15(r, acc)
            }
            204 => {
                acc.push('í');
                s15(r, acc)
            }
            205 => {
                acc.push('î');
                s15(r, acc)
            }
            206 => {
                acc.push('ï');
                s15(r, acc)
            }
            207 => {
                acc.push('ð');
                s15(r, acc)
            }
            208 => {
                acc.push('ñ');
                s15(r, acc)
            }
            209 => {
                acc.push('ò');
                s15(r, acc)
            }
            210 => {
                acc.push('ó');
                s15(r, acc)
            }
            211 => {
                acc.push('ô');
                s15(r, acc)
            }
            212 => {
                acc.push('õ');
                s15(r, acc)
            }
            213 => {
                acc.push('ö');
                s15(r, acc)
            }
            214 => {
                acc.push('÷');
                s15(r, acc)
            }
            215 => {
                acc.push('ø');
                s15(r, acc)
            }
            216 => {
                acc.push('ù');
                s15(r, acc)
            }
            217 => {
                acc.push('ú');
                s15(r, acc)
            }
            218 => {
                acc.push('û');
                s15(r, acc)
            }
            219 => {
                acc.push('ü');
                s15(r, acc)
            }
            220 => {
                acc.push('ý');
                s15(r, acc)
            }
            221 => {
                acc.push('þ');
                s15(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: '"' -> '\\'
    pub fn s16<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        acc.push('\\');
        s15(r, acc)
    }
    #[inline]
    ///Minimal input to reach this state: 'e' -> 'u' -> 'r'
    pub fn s17<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        acc.push('t');
        s0(r, acc)
    }
    #[inline]
    ///Minimal input to reach this state: 'e' -> 'u'
    pub fn s18<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        acc.push('r');
        s17(r, acc)
    }
    #[inline]
    ///Minimal input to reach this state: 'e'
    pub fn s19<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        acc.push('u');
        s18(r, acc)
    }
    #[inline]
    ///Minimal input to reach this state: 'l' -> 'l' -> 'u'
    pub fn s20<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        acc.push('n');
        s0(r, acc)
    }
    #[inline]
    ///Minimal input to reach this state: 'l' -> 'l'
    pub fn s21<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        acc.push('u');
        s20(r, acc)
    }
    #[inline]
    ///Minimal input to reach this state: 'l'
    pub fn s22<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        acc.push('l');
        s21(r, acc)
    }
    #[inline]
    ///Minimal input to reach this state: ':' -> '"' -> '"'
    pub fn s23<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..3) {
            0 => {
                acc.push('\n');
                s24(r, acc)
            }
            1 => {
                acc.push(' ');
                s23(r, acc)
            }
            2 => {
                acc.push('{');
                s0(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: ':' -> '"' -> '"' -> '\n'
    pub fn s24<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..4) {
            0 => {
                acc.push('\n');
                s24(r, acc)
            }
            1 => {
                acc.push('\r');
                s23(r, acc)
            }
            2 => {
                acc.push(' ');
                s23(r, acc)
            }
            3 => {
                acc.push('{');
                s0(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: ':' -> '"'
    pub fn s25<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..222) {
            0 => {
                acc.push(' ');
                s25(r, acc)
            }
            1 => {
                acc.push('!');
                s25(r, acc)
            }
            2 => {
                acc.push('"');
                s23(r, acc)
            }
            3 => {
                acc.push('#');
                s25(r, acc)
            }
            4 => {
                acc.push('$');
                s25(r, acc)
            }
            5 => {
                acc.push('%');
                s25(r, acc)
            }
            6 => {
                acc.push('&');
                s25(r, acc)
            }
            7 => {
                acc.push('\'');
                s25(r, acc)
            }
            8 => {
                acc.push('(');
                s25(r, acc)
            }
            9 => {
                acc.push(')');
                s25(r, acc)
            }
            10 => {
                acc.push('*');
                s25(r, acc)
            }
            11 => {
                acc.push('+');
                s25(r, acc)
            }
            12 => {
                acc.push(',');
                s25(r, acc)
            }
            13 => {
                acc.push('-');
                s25(r, acc)
            }
            14 => {
                acc.push('.');
                s25(r, acc)
            }
            15 => {
                acc.push('/');
                s25(r, acc)
            }
            16 => {
                acc.push('0');
                s25(r, acc)
            }
            17 => {
                acc.push('1');
                s25(r, acc)
            }
            18 => {
                acc.push('2');
                s25(r, acc)
            }
            19 => {
                acc.push('3');
                s25(r, acc)
            }
            20 => {
                acc.push('4');
                s25(r, acc)
            }
            21 => {
                acc.push('5');
                s25(r, acc)
            }
            22 => {
                acc.push('6');
                s25(r, acc)
            }
            23 => {
                acc.push('7');
                s25(r, acc)
            }
            24 => {
                acc.push('8');
                s25(r, acc)
            }
            25 => {
                acc.push('9');
                s25(r, acc)
            }
            26 => {
                acc.push(':');
                s25(r, acc)
            }
            27 => {
                acc.push(';');
                s25(r, acc)
            }
            28 => {
                acc.push('<');
                s25(r, acc)
            }
            29 => {
                acc.push('=');
                s25(r, acc)
            }
            30 => {
                acc.push('>');
                s25(r, acc)
            }
            31 => {
                acc.push('?');
                s25(r, acc)
            }
            32 => {
                acc.push('@');
                s25(r, acc)
            }
            33 => {
                acc.push('A');
                s25(r, acc)
            }
            34 => {
                acc.push('B');
                s25(r, acc)
            }
            35 => {
                acc.push('C');
                s25(r, acc)
            }
            36 => {
                acc.push('D');
                s25(r, acc)
            }
            37 => {
                acc.push('E');
                s25(r, acc)
            }
            38 => {
                acc.push('F');
                s25(r, acc)
            }
            39 => {
                acc.push('G');
                s25(r, acc)
            }
            40 => {
                acc.push('H');
                s25(r, acc)
            }
            41 => {
                acc.push('I');
                s25(r, acc)
            }
            42 => {
                acc.push('J');
                s25(r, acc)
            }
            43 => {
                acc.push('K');
                s25(r, acc)
            }
            44 => {
                acc.push('L');
                s25(r, acc)
            }
            45 => {
                acc.push('M');
                s25(r, acc)
            }
            46 => {
                acc.push('N');
                s25(r, acc)
            }
            47 => {
                acc.push('O');
                s25(r, acc)
            }
            48 => {
                acc.push('P');
                s25(r, acc)
            }
            49 => {
                acc.push('Q');
                s25(r, acc)
            }
            50 => {
                acc.push('R');
                s25(r, acc)
            }
            51 => {
                acc.push('S');
                s25(r, acc)
            }
            52 => {
                acc.push('T');
                s25(r, acc)
            }
            53 => {
                acc.push('U');
                s25(r, acc)
            }
            54 => {
                acc.push('V');
                s25(r, acc)
            }
            55 => {
                acc.push('W');
                s25(r, acc)
            }
            56 => {
                acc.push('X');
                s25(r, acc)
            }
            57 => {
                acc.push('Y');
                s25(r, acc)
            }
            58 => {
                acc.push('Z');
                s25(r, acc)
            }
            59 => {
                acc.push('[');
                s25(r, acc)
            }
            60 => {
                acc.push('\\');
                s26(r, acc)
            }
            61 => {
                acc.push(']');
                s25(r, acc)
            }
            62 => {
                acc.push('^');
                s25(r, acc)
            }
            63 => {
                acc.push('_');
                s25(r, acc)
            }
            64 => {
                acc.push('`');
                s25(r, acc)
            }
            65 => {
                acc.push('a');
                s25(r, acc)
            }
            66 => {
                acc.push('b');
                s25(r, acc)
            }
            67 => {
                acc.push('c');
                s25(r, acc)
            }
            68 => {
                acc.push('d');
                s25(r, acc)
            }
            69 => {
                acc.push('e');
                s25(r, acc)
            }
            70 => {
                acc.push('f');
                s25(r, acc)
            }
            71 => {
                acc.push('g');
                s25(r, acc)
            }
            72 => {
                acc.push('h');
                s25(r, acc)
            }
            73 => {
                acc.push('i');
                s25(r, acc)
            }
            74 => {
                acc.push('j');
                s25(r, acc)
            }
            75 => {
                acc.push('k');
                s25(r, acc)
            }
            76 => {
                acc.push('l');
                s25(r, acc)
            }
            77 => {
                acc.push('m');
                s25(r, acc)
            }
            78 => {
                acc.push('n');
                s25(r, acc)
            }
            79 => {
                acc.push('o');
                s25(r, acc)
            }
            80 => {
                acc.push('p');
                s25(r, acc)
            }
            81 => {
                acc.push('q');
                s25(r, acc)
            }
            82 => {
                acc.push('r');
                s25(r, acc)
            }
            83 => {
                acc.push('s');
                s25(r, acc)
            }
            84 => {
                acc.push('t');
                s25(r, acc)
            }
            85 => {
                acc.push('u');
                s25(r, acc)
            }
            86 => {
                acc.push('v');
                s25(r, acc)
            }
            87 => {
                acc.push('w');
                s25(r, acc)
            }
            88 => {
                acc.push('x');
                s25(r, acc)
            }
            89 => {
                acc.push('y');
                s25(r, acc)
            }
            90 => {
                acc.push('z');
                s25(r, acc)
            }
            91 => {
                acc.push('{');
                s25(r, acc)
            }
            92 => {
                acc.push('|');
                s25(r, acc)
            }
            93 => {
                acc.push('}');
                s25(r, acc)
            }
            94 => {
                acc.push('~');
                s25(r, acc)
            }
            95 => {
                acc.push('\u{80}');
                s25(r, acc)
            }
            96 => {
                acc.push('\u{81}');
                s25(r, acc)
            }
            97 => {
                acc.push('\u{82}');
                s25(r, acc)
            }
            98 => {
                acc.push('\u{83}');
                s25(r, acc)
            }
            99 => {
                acc.push('\u{84}');
                s25(r, acc)
            }
            100 => {
                acc.push('\u{85}');
                s25(r, acc)
            }
            101 => {
                acc.push('\u{86}');
                s25(r, acc)
            }
            102 => {
                acc.push('\u{87}');
                s25(r, acc)
            }
            103 => {
                acc.push('\u{88}');
                s25(r, acc)
            }
            104 => {
                acc.push('\u{89}');
                s25(r, acc)
            }
            105 => {
                acc.push('\u{8a}');
                s25(r, acc)
            }
            106 => {
                acc.push('\u{8b}');
                s25(r, acc)
            }
            107 => {
                acc.push('\u{8c}');
                s25(r, acc)
            }
            108 => {
                acc.push('\u{8d}');
                s25(r, acc)
            }
            109 => {
                acc.push('\u{8e}');
                s25(r, acc)
            }
            110 => {
                acc.push('\u{8f}');
                s25(r, acc)
            }
            111 => {
                acc.push('\u{90}');
                s25(r, acc)
            }
            112 => {
                acc.push('\u{91}');
                s25(r, acc)
            }
            113 => {
                acc.push('\u{92}');
                s25(r, acc)
            }
            114 => {
                acc.push('\u{93}');
                s25(r, acc)
            }
            115 => {
                acc.push('\u{94}');
                s25(r, acc)
            }
            116 => {
                acc.push('\u{95}');
                s25(r, acc)
            }
            117 => {
                acc.push('\u{96}');
                s25(r, acc)
            }
            118 => {
                acc.push('\u{97}');
                s25(r, acc)
            }
            119 => {
                acc.push('\u{98}');
                s25(r, acc)
            }
            120 => {
                acc.push('\u{99}');
                s25(r, acc)
            }
            121 => {
                acc.push('\u{9a}');
                s25(r, acc)
            }
            122 => {
                acc.push('\u{9b}');
                s25(r, acc)
            }
            123 => {
                acc.push('\u{9c}');
                s25(r, acc)
            }
            124 => {
                acc.push('\u{9d}');
                s25(r, acc)
            }
            125 => {
                acc.push('\u{9e}');
                s25(r, acc)
            }
            126 => {
                acc.push('\u{9f}');
                s25(r, acc)
            }
            127 => {
                acc.push('\u{a0}');
                s25(r, acc)
            }
            128 => {
                acc.push('¡');
                s25(r, acc)
            }
            129 => {
                acc.push('¢');
                s25(r, acc)
            }
            130 => {
                acc.push('£');
                s25(r, acc)
            }
            131 => {
                acc.push('¤');
                s25(r, acc)
            }
            132 => {
                acc.push('¥');
                s25(r, acc)
            }
            133 => {
                acc.push('¦');
                s25(r, acc)
            }
            134 => {
                acc.push('§');
                s25(r, acc)
            }
            135 => {
                acc.push('¨');
                s25(r, acc)
            }
            136 => {
                acc.push('©');
                s25(r, acc)
            }
            137 => {
                acc.push('ª');
                s25(r, acc)
            }
            138 => {
                acc.push('«');
                s25(r, acc)
            }
            139 => {
                acc.push('¬');
                s25(r, acc)
            }
            140 => {
                acc.push('\u{ad}');
                s25(r, acc)
            }
            141 => {
                acc.push('®');
                s25(r, acc)
            }
            142 => {
                acc.push('¯');
                s25(r, acc)
            }
            143 => {
                acc.push('°');
                s25(r, acc)
            }
            144 => {
                acc.push('±');
                s25(r, acc)
            }
            145 => {
                acc.push('²');
                s25(r, acc)
            }
            146 => {
                acc.push('³');
                s25(r, acc)
            }
            147 => {
                acc.push('´');
                s25(r, acc)
            }
            148 => {
                acc.push('µ');
                s25(r, acc)
            }
            149 => {
                acc.push('¶');
                s25(r, acc)
            }
            150 => {
                acc.push('·');
                s25(r, acc)
            }
            151 => {
                acc.push('¸');
                s25(r, acc)
            }
            152 => {
                acc.push('¹');
                s25(r, acc)
            }
            153 => {
                acc.push('º');
                s25(r, acc)
            }
            154 => {
                acc.push('»');
                s25(r, acc)
            }
            155 => {
                acc.push('¼');
                s25(r, acc)
            }
            156 => {
                acc.push('½');
                s25(r, acc)
            }
            157 => {
                acc.push('¾');
                s25(r, acc)
            }
            158 => {
                acc.push('¿');
                s25(r, acc)
            }
            159 => {
                acc.push('À');
                s25(r, acc)
            }
            160 => {
                acc.push('Á');
                s25(r, acc)
            }
            161 => {
                acc.push('Â');
                s25(r, acc)
            }
            162 => {
                acc.push('Ã');
                s25(r, acc)
            }
            163 => {
                acc.push('Ä');
                s25(r, acc)
            }
            164 => {
                acc.push('Å');
                s25(r, acc)
            }
            165 => {
                acc.push('Æ');
                s25(r, acc)
            }
            166 => {
                acc.push('Ç');
                s25(r, acc)
            }
            167 => {
                acc.push('È');
                s25(r, acc)
            }
            168 => {
                acc.push('É');
                s25(r, acc)
            }
            169 => {
                acc.push('Ê');
                s25(r, acc)
            }
            170 => {
                acc.push('Ë');
                s25(r, acc)
            }
            171 => {
                acc.push('Ì');
                s25(r, acc)
            }
            172 => {
                acc.push('Í');
                s25(r, acc)
            }
            173 => {
                acc.push('Î');
                s25(r, acc)
            }
            174 => {
                acc.push('Ï');
                s25(r, acc)
            }
            175 => {
                acc.push('Ð');
                s25(r, acc)
            }
            176 => {
                acc.push('Ñ');
                s25(r, acc)
            }
            177 => {
                acc.push('Ò');
                s25(r, acc)
            }
            178 => {
                acc.push('Ó');
                s25(r, acc)
            }
            179 => {
                acc.push('Ô');
                s25(r, acc)
            }
            180 => {
                acc.push('Õ');
                s25(r, acc)
            }
            181 => {
                acc.push('Ö');
                s25(r, acc)
            }
            182 => {
                acc.push('×');
                s25(r, acc)
            }
            183 => {
                acc.push('Ø');
                s25(r, acc)
            }
            184 => {
                acc.push('Ù');
                s25(r, acc)
            }
            185 => {
                acc.push('Ú');
                s25(r, acc)
            }
            186 => {
                acc.push('Û');
                s25(r, acc)
            }
            187 => {
                acc.push('Ü');
                s25(r, acc)
            }
            188 => {
                acc.push('Ý');
                s25(r, acc)
            }
            189 => {
                acc.push('Þ');
                s25(r, acc)
            }
            190 => {
                acc.push('ß');
                s25(r, acc)
            }
            191 => {
                acc.push('à');
                s25(r, acc)
            }
            192 => {
                acc.push('á');
                s25(r, acc)
            }
            193 => {
                acc.push('â');
                s25(r, acc)
            }
            194 => {
                acc.push('ã');
                s25(r, acc)
            }
            195 => {
                acc.push('ä');
                s25(r, acc)
            }
            196 => {
                acc.push('å');
                s25(r, acc)
            }
            197 => {
                acc.push('æ');
                s25(r, acc)
            }
            198 => {
                acc.push('ç');
                s25(r, acc)
            }
            199 => {
                acc.push('è');
                s25(r, acc)
            }
            200 => {
                acc.push('é');
                s25(r, acc)
            }
            201 => {
                acc.push('ê');
                s25(r, acc)
            }
            202 => {
                acc.push('ë');
                s25(r, acc)
            }
            203 => {
                acc.push('ì');
                s25(r, acc)
            }
            204 => {
                acc.push('í');
                s25(r, acc)
            }
            205 => {
                acc.push('î');
                s25(r, acc)
            }
            206 => {
                acc.push('ï');
                s25(r, acc)
            }
            207 => {
                acc.push('ð');
                s25(r, acc)
            }
            208 => {
                acc.push('ñ');
                s25(r, acc)
            }
            209 => {
                acc.push('ò');
                s25(r, acc)
            }
            210 => {
                acc.push('ó');
                s25(r, acc)
            }
            211 => {
                acc.push('ô');
                s25(r, acc)
            }
            212 => {
                acc.push('õ');
                s25(r, acc)
            }
            213 => {
                acc.push('ö');
                s25(r, acc)
            }
            214 => {
                acc.push('÷');
                s25(r, acc)
            }
            215 => {
                acc.push('ø');
                s25(r, acc)
            }
            216 => {
                acc.push('ù');
                s25(r, acc)
            }
            217 => {
                acc.push('ú');
                s25(r, acc)
            }
            218 => {
                acc.push('û');
                s25(r, acc)
            }
            219 => {
                acc.push('ü');
                s25(r, acc)
            }
            220 => {
                acc.push('ý');
                s25(r, acc)
            }
            221 => {
                acc.push('þ');
                s25(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: ':' -> '"' -> '\\'
    pub fn s26<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        acc.push('\\');
        s25(r, acc)
    }
    #[inline]
    ///Minimal input to reach this state: ':'
    pub fn s27<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..3) {
            0 => {
                acc.push('\n');
                s28(r, acc)
            }
            1 => {
                acc.push(' ');
                s27(r, acc)
            }
            2 => {
                acc.push('"');
                s25(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
    #[inline]
    ///Minimal input to reach this state: ':' -> '\n'
    pub fn s28<R: rand::Rng>(r: &mut R, mut acc: Vec<char>) -> Vec<char> {
        match r.gen_range(0..4) {
            0 => {
                acc.push('\n');
                s28(r, acc)
            }
            1 => {
                acc.push('\r');
                s27(r, acc)
            }
            2 => {
                acc.push(' ');
                s27(r, acc)
            }
            3 => {
                acc.push('"');
                s25(r, acc)
            }
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }
}
