//! Automatically generated with [inator](https://crates.io/crates/inator).

#![allow(dead_code, unused_variables)]

/// Descriptive parsing error.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    /// Token without any relevant rule.
    Absurd {
        /// Index of the token that caused this error.
        index: usize,
        /// Particular token that didn't correspond to a rule.
        token: u8,
    },
    /// Token that would have closed a delimiter, but the delimiter wasn't open.
    Unopened {
        /// What was actually open, if anything, and the index of the token that opened it.
        what_was_open: Option<(&'static str, usize)>,
        /// Index of the token that caused this error.
        index: usize,
    },
    /// After parsing all input, a delimiter remains open (e.g. "(a, b, c").
    Unclosed {
        /// Region (user-defined name) that was not closed. Sensible to be e.g. "parentheses" for `(...)`.
        region: &'static str,
        /// Index at which the delimiter was opened (e.g., for parentheses, the index of the relevant '(').
        opened: usize,
    },
    /// Ended on a user-defined non-accepting state.
    UserDefined {
        /// User-defined error message.
        messages: &'static [&'static str],
    },
}

type R<I> = Result<(Option<(usize, Option<F<I>>)>, ()), Error>;

#[repr(transparent)]
struct F<I>(fn(&mut I, ()) -> R<I>);

#[inline]
pub fn parse<I: IntoIterator<Item = u8>>(input: I) -> Result<(), Error> {
    state_1(&mut input.into_iter().enumerate(), (), None)
}

#[inline]
fn state_0<I: Iterator<Item = (usize, u8)>>(
    input: &mut I,
    acc: (),
    stack_top: Option<(&'static str, usize)>,
) -> Result<(), Error> {
    match input.next() {
        None => stack_top.map_or(Ok(acc), |(region, opened)| {
            Err(Error::Unclosed { region, opened })
        }),
        Some((index, token)) => match token {
            _ => Err(Error::Absurd { index, token }),
        },
    }
}

#[inline]
fn state_1<I: Iterator<Item = (usize, u8)>>(
    input: &mut I,
    acc: (),
    stack_top: Option<(&'static str, usize)>,
) -> Result<(), Error> {
    match input.next() {
        None => Err(Error::UserDefined {
            messages: &[
                "Expected a token in the range [b\' \'..=b\' \'] but input ended",
                "Expected a token in the range [b\'\\n\'..=b\'\\n\'] but input ended",
                "Expected a token in the range [b\'\\r\'..=b\'\\r\'] but input ended",
                "Expected a token in the range [b\'\\t\'..=b\'\\t\'] but input ended",
            ],
        }),
        Some((index, token)) => match token {
            b'\t'..=b'\t' => state_0(input, (|(), _| {})(acc, token), stack_top),
            b'\n'..=b'\n' => state_0(input, (|(), _| {})(acc, token), stack_top),
            b'\r'..=b'\r' => state_0(input, (|(), _| {})(acc, token), stack_top),
            b' '..=b' ' => state_0(input, (|(), _| {})(acc, token), stack_top),
            _ => Err(Error::Absurd { index, token }),
        },
    }
}
