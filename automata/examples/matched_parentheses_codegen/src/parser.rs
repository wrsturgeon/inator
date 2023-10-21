/// Descriptive parsing error.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    /// Token without any relevant rule.
    Absurd {
        /// Index of the token that caused this error.
        index: usize,
        /// Particular token that didn't correspond to a rule.
        token: char,
    },
    /// Token that would have closed a delimiter, but the delimiter wasn't open.
    Unopened {
        /// Index of the token that caused this error.
        index: usize,
        /// Type of thing that wasn't opened (e.g. parentheses).
        delimiter: symbols::Symbol,
        /// What actually was open (e.g. you tried to close parentheses, but a bracket was open).
        instead: Option<symbols::Symbol>,
    },
    /// After parsing all input, a delimiter remains open (e.g. "(a, b, c").
    Unclosed {
        /// Index at which the delimiter was opened (e.g., for parentheses, the index of the relevant '(').
        opened: usize,
        /// Type of thing that wasn't closed (e.g. parentheses).
        delimiter: symbols::Symbol,
    },
}

type R<I> = Result<(Option<(usize, symbols::Symbol, Option<F<I>>)>, ()), Error>;

#[repr(transparent)]
struct F<I>(fn(&mut I, Option<symbols::Symbol>, ()) -> R<I>);

#[inline]
pub fn parse<I: IntoIterator<Item = char>>(input: I) -> Result<(), Error> {
    match state_0(
        &mut input.into_iter().enumerate(),
        None,
        <() as Default>::default(),
    )? {
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
fn state_0<I: Iterator<Item = (usize, char)>>(
    input: &mut I,
    context: Option<symbols::Symbol>,
    acc: (),
) -> R<I> {
    match input.next() {
        None => Ok((None, acc)),
        Some((index, token)) => match (&context, &token) {
            (&_, &('('..='(')) => match state_0(
                input,
                Some(symbols::Symbol::Paren),
                (|(), _| ())(acc, token),
            )? {
                (None | Some((_, _, None)), _) => Err(Error::Unclosed {
                    opened: index,
                    delimiter: symbols::Symbol::Paren,
                }),
                (Some((idx, ctx, Some(F(f)))), out) => f(input, Some(ctx), out),
            },
            (&Some(symbols::Symbol::Paren), &(')'..=')')) => Ok((
                Some((index, symbols::Symbol::Paren, Some(F(state_0)))),
                (|(), _| ())(acc, token),
            )),
            _ => Err(Error::Absurd { index, token }),
        },
    }
}
