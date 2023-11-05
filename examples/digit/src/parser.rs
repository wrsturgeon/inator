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
        /// Index of the token that caused this error.
        index: usize,
        /// Type of thing that wasn't opened (e.g. parentheses).
        delimiter: (),
        /// What actually was open (e.g. you tried to close parentheses, but a bracket was open).
        instead: Option<()>,
    },
    /// After parsing all input, a delimiter remains open (e.g. "(a, b, c").
    Unclosed {
        /// Index at which the delimiter was opened (e.g., for parentheses, the index of the relevant '(').
        opened: usize,
        /// Type of thing that wasn't closed (e.g. parentheses).
        delimiter: (),
    },
    /// Ended on a user-defined non-accepting state.
    UserDefined {
        /// User-defined error message.
        messages: &'static [&'static str],
    },
}

type R<I> = Result<(Option<(usize, (), Option<F<I>>)>, u8), Error>;

#[repr(transparent)]
struct F<I>(fn(&mut I, Option<()>, u8) -> R<I>);

#[inline]
pub fn parse<I: IntoIterator<Item = u8>>(input: I) -> Result<u8, Error> {
    match state_1(&mut input.into_iter().enumerate(), None, Default::default())? {
        (None, out) => Ok(out),
        (Some((index, context, None)), out) => panic!("Some(({index:?}, {context:?}, None))"),
        (Some((index, delimiter, Some(F(_)))), _) => Err(Error::Unopened {
            index,
            delimiter,
            instead: None,
        }),
    }
}

#[inline]
fn state_0<I: Iterator<Item = (usize, u8)>>(input: &mut I, context: Option<()>, acc: u8) -> R<I> {
    match input.next() {
        None => Ok((None, acc)),
        Some((index, token)) => match (&context, &token) {
            _ => Err(Error::Absurd { index, token }),
        },
    }
}

#[inline]
fn state_1<I: Iterator<Item = (usize, u8)>>(input: &mut I, context: Option<()>, acc: ()) -> R<I> {
    match input.next() {
        None => Err(Error::UserDefined {
            messages: &["Expected a token in the range [b\'0\'..=b\'9\'] but input ended"],
        }),
        Some((index, token)) => match (&context, &token) {
            (&_, &(b'0'..=b'9')) => {
                match state_0(input, context, (|(), i| i - b'0')(acc, token))? {
                    (done @ (None | Some((_, _, None))), acc) => Ok((done, acc)),
                    (Some((idx, ctx, Some(F(f)))), out) => f(input, Some(ctx), out),
                }
            }
            _ => Err(Error::Absurd { index, token }),
        },
    }
}
