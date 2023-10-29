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
        delimiter: types::Stack,
        /// What actually was open (e.g. you tried to close parentheses, but a bracket was open).
        instead: Option<types::Stack>,
    },
    /// After parsing all input, a delimiter remains open (e.g. "(a, b, c").
    Unclosed {
        /// Index at which the delimiter was opened (e.g., for parentheses, the index of the relevant '(').
        opened: usize,
        /// Type of thing that wasn't closed (e.g. parentheses).
        delimiter: types::Stack,
    },
    /// Ended on a user-defined non-accepting state.
    UserDefined {
        /// User-defined error message.
        message: &'static str,
    },
}

type R<I> = Result<(Option<(usize, types::Stack, Option<F<I>>)>, ()), Error>;

#[repr(transparent)]
struct F<I>(fn(&mut I, Option<types::Stack>, ()) -> R<I>);

#[inline]
pub fn parse<I: IntoIterator<Item = u8>>(input: I) -> Result<(), Error> {
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
fn state_0<I: Iterator<Item = (usize, u8)>>(
    input: &mut I,
    context: Option<types::Stack>,
    acc: (),
) -> R<I> {
    match input.next() {
        None => Ok((None, acc)),
        Some((index, token)) => match (&context, &token) {
            _ => Err(Error::Absurd { index, token }),
        },
    }
}

#[inline]
fn state_1<I: Iterator<Item = (usize, u8)>>(
    input: &mut I,
    context: Option<types::Stack>,
    acc: (),
) -> R<I> {
    match input.next() {
        None => Err(Error::UserDefined {
            message:
                "Expected only a single token on [b'\t'..=b'\t'] but got another token after it",
        }),
        Some((index, token)) => match (&context, &token) {
            (&_, &(b'\t'..=b'\t')) => match state_0(input, context, (|(), _| {})(acc, token))? {
                (None, _) => todo!(),
                (Some((_, _, None)), acc) => Ok(acc),
                (Some((idx, ctx, Some(F(f)))), out) => f(input, Some(ctx), out),
            },
            (&_, &(b'\n'..=b'\n')) => match state_0(input, context, (|(), _| {})(acc, token))? {
                (None, _) => todo!(),
                (Some((_, _, None)), acc) => Ok(acc),
                (Some((idx, ctx, Some(F(f)))), out) => f(input, Some(ctx), out),
            },
            (&_, &(b'\r'..=b'\r')) => match state_0(input, context, (|(), _| {})(acc, token))? {
                (None, _) => todo!(),
                (Some((_, _, None)), acc) => Ok(acc),
                (Some((idx, ctx, Some(F(f)))), out) => f(input, Some(ctx), out),
            },
            (&_, &(b' '..=b' ')) => match state_0(input, context, (|(), _| {})(acc, token))? {
                (None, _) => todo!(),
                (Some((_, _, None)), acc) => Ok(acc),
                (Some((idx, ctx, Some(F(f)))), out) => f(input, Some(ctx), out),
            },
            _ => Err(Error::Absurd { index, token }),
        },
    }
}
