/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Translate an automaton into Rust source code.

use crate::{
    Action, CurryInput, CurryStack, Graph, Input, Range, RangeMap, Stack, State, Transition,
};
use core::borrow::Borrow;
use std::collections::BTreeSet;

/// Translate a value into Rust source code that reproduces it.
pub trait ToSrc: 'static {
    /// Translate a value into Rust source code that reproduces it.
    #[must_use]
    fn to_src(&self) -> String;
    /// Translate a type into Rust source code that reproduces it.
    #[must_use]
    fn src_type() -> String;
}

impl ToSrc for () {
    #[inline(always)]
    #[must_use]
    fn to_src(&self) -> String {
        Self::src_type()
    }
    #[inline]
    #[must_use]
    fn src_type() -> String {
        "()".to_owned()
    }
}

impl ToSrc for usize {
    #[inline]
    #[must_use]
    fn to_src(&self) -> String {
        format!("{self}")
    }
    #[inline]
    #[must_use]
    fn src_type() -> String {
        "usize".to_owned()
    }
}

impl ToSrc for u8 {
    #[inline]
    #[must_use]
    fn to_src(&self) -> String {
        format!("{self}")
    }
    #[inline]
    #[must_use]
    fn src_type() -> String {
        "u8".to_owned()
    }
}

impl ToSrc for char {
    #[inline]
    #[must_use]
    fn to_src(&self) -> String {
        format!("'{}'", self.escape_default())
    }
    #[inline]
    #[must_use]
    fn src_type() -> String {
        "char".to_owned()
    }
}

impl<T: ToSrc> ToSrc for BTreeSet<T> {
    #[inline]
    #[must_use]
    fn to_src(&self) -> String {
        let mut iter = self.iter();
        let Some(fst) = iter.next() else {
            return format!("{}::new()", Self::src_type());
        };
        let Some(snd) = iter.next() else {
            return format!("core::iter::once({}).collect()", fst.to_src());
        };
        format!(
            "[{}, {}{}].collect()",
            fst.to_src(),
            snd.to_src(),
            iter.fold(String::new(), |acc, x| format!("{acc}, {}", x.to_src())),
        )
    }
    #[inline]
    #[must_use]
    fn src_type() -> String {
        "std::collections::BTreeSet::<usize>".to_owned()
    }
}

impl<T: ToSrc> ToSrc for Option<T> {
    #[inline]
    #[must_use]
    fn to_src(&self) -> String {
        self.as_ref()
            .map_or_else(|| "None".to_owned(), |x| format!("Some({})", x.to_src()))
    }
    #[inline]
    #[must_use]
    fn src_type() -> String {
        format!("Option::<{}>", T::src_type())
    }
}

impl<T: ToSrc, E: ToSrc> ToSrc for Result<T, E> {
    #[inline]
    #[must_use]
    fn to_src(&self) -> String {
        self.as_ref().map_or_else(
            |e| format!("Err({})", e.to_src()),
            |x| format!("Some({})", x.to_src()),
        )
    }
    #[inline]
    #[must_use]
    fn src_type() -> String {
        format!("Result::<{}, {}>", T::src_type(), E::src_type())
    }
}

impl ToSrc for String {
    #[inline]
    #[must_use]
    fn to_src(&self) -> String {
        format!("\"{self}\".to_owned()")
    }
    #[inline]
    #[must_use]
    fn src_type() -> String {
        "String".to_owned()
    }
}

impl<T: Clone + Ord + ToSrc> ToSrc for Range<T> {
    #[inline]
    #[must_use]
    fn to_src(&self) -> String {
        format!("{}..={}", self.first.to_src(), self.last.to_src())
    }
    #[inline(always)]
    #[must_use]
    fn src_type() -> String {
        T::src_type()
    }
}

impl<I: Input, S: Stack> Graph<I, S, usize> {
    /// Translate a value into Rust source code that reproduces it.
    #[inline]
    #[must_use]
    #[allow(clippy::arithmetic_side_effects)] // <-- String concatenation with `+`
    pub fn to_src(&self) -> String {
        let input_t = I::src_type();
        let output_t: &str = &self.output_t;
        let stack_t = S::src_type();
        format!(
            r#"/// Descriptive parsing error.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum Error {{
    /// Token without any relevant rule.
    Absurd {{
        /// Index of the token that caused this error.
        index: usize,
        /// Particular token that didn't correspond to a rule.
        token: {input_t},
    }},
    /// Token that would have closed a delimiter, but the delimiter wasn't open.
    Unopened {{
        /// Index of the token that caused this error.
        index: usize,
        /// Type of thing that wasn't opened (e.g. parentheses).
        delimiter: {stack_t},
        /// What actually was open (e.g. you tried to close parentheses, but a bracket was open).
        instead: Option<{stack_t}>,
    }},
    /// After parsing all input, a delimiter remains open (e.g. "(a, b, c").
    Unclosed {{
        /// Index at which the delimiter was opened (e.g., for parentheses, the index of the relevant '(').
        opened: usize,
        /// Type of thing that wasn't closed (e.g. parentheses).
        delimiter: {stack_t},
    }},
}}

type R<I> = Result<(Option<(usize, {stack_t}, Option<F<I>>)>, {output_t}), Error>;

#[repr(transparent)]
struct F<I>(fn(&mut I, Option<{stack_t}>, {output_t}) -> R<I>);

#[inline]
pub fn parse<I: IntoIterator<Item = {input_t}>>(input: I) -> Result<{output_t}, Error> {{
    match state_{}(
        &mut input.into_iter().enumerate(),
        None,
        <{output_t} as Default>::default(),
    )? {{
        (None, out) => Ok(out),
        (Some((index, context, None)), out) => panic!("Some(({{index:?}}, {{context:?}}, None))"),
        (Some((index, delimiter, Some(F(_)))), _) => Err(Error::Unopened {{ index, delimiter, instead: None, }}),
    }}
}}{}"#,
            self.initial,
            self.states
                .iter()
                .enumerate()
                .fold(String::new(), |acc, (i, s)| acc + &s.to_src(i)),
        )
    }
}

impl<I: Input, S: Stack> State<I, S, usize> {
    /// Translate a value into Rust source code that reproduces it.
    #[inline]
    #[must_use]
    fn to_src(&self, i: usize) -> String {
        let input_t: &str = &self.input_t;
        format!(
            r#"


#[inline]
fn state_{i}<I: Iterator<Item = (usize, {})>>(input: &mut I, context: Option<{}>, acc: {input_t}) -> R<I> {{
    match input.next() {{
        None => {},
        Some((index, token)) => {},
    }}
}}"#,
            I::src_type(),
            S::src_type(),
            if self.accepting {
                "Ok((None, acc))"
            } else {
                "Err(TODO_IMPLEMENTATION_DEFINED)"
            },
            self.transitions.to_src(),
        )
    }
}

impl<I: Input, S: Stack> CurryStack<I, S, usize> {
    /// Translate a value into Rust source code that reproduces it.
    #[inline]
    #[must_use]
    #[allow(clippy::arithmetic_side_effects)] // <-- string concatenation with `+`
    fn to_src(&self) -> String {
        format!(
            r#"match (&context, &token) {{{}
            _ => Err(Error::Absurd {{ index, token }}),
        }}"#,
            self.wildcard
                .iter()
                .map(|v| (None, v))
                .chain(self.map_none.iter().map(|v| (Some(None), v)))
                .chain(
                    self.map_some
                        .iter()
                        .map(|(k, v)| (Some(Some(k.to_src())), v))
                )
                .fold(String::new(), |acc, (k, v)| acc
                    + &v.to_src(
                        k.as_ref().map(|opt| opt.as_ref().map(Borrow::borrow))
                    ))
        )
    }
}

impl<I: Input, S: Stack> CurryInput<I, S, usize> {
    /// Translate a value into Rust source code that reproduces it.
    #[inline]
    #[must_use]
    fn to_src(&self, stack_symbol: Option<Option<&str>>) -> String {
        let s = stack_symbol.map_or_else(
            || "_".to_owned(),
            |sym| sym.map_or_else(|| "None".to_owned(), |x| format!("Some({x})")),
        );
        match *self {
            Self::Wildcard(ref etc) => format!(
                r#"
            (&{s}, _) => {},"#,
                etc.to_src(stack_symbol),
            ),
            Self::Scrutinize(ref etc) => etc.to_src(stack_symbol),
        }
    }
}

impl<I: Input, S: Stack> RangeMap<I, S, usize> {
    /// Translate a value into Rust source code that reproduces it.
    #[inline]
    #[must_use]
    fn to_src(&self, stack_symbol: Option<Option<&str>>) -> String {
        let s = stack_symbol.map_or_else(
            || "_".to_owned(),
            |sym| sym.map_or_else(|| "None".to_owned(), |x| format!("Some({x})")),
        );
        self.entries.iter().fold(String::new(), |acc, (k, v)| {
            format!(
                r#"{acc}
            (&{s}, &({})) => {},"#,
                k.to_src(),
                v.to_src(stack_symbol),
            )
        })
    }
}

impl<I: Input, S: Stack> Transition<I, S, usize> {
    /// Translate a value into Rust source code that reproduces it.
    #[inline]
    #[must_use]
    #[allow(clippy::todo)] // TODO: what the fuck does the last case mean?
    fn to_src(&self, stack_symbol: Option<Option<&str>>) -> String {
        let dst = self.dst;
        let f = self.update.src;
        match self.act {
            Action::Local => format!(
                r#"match state_{dst}(input, context, ({f})(acc, token))? {{
                (None, _) => todo!(),
                (Some((_, _, None)), acc) => Ok(acc),
                (Some((idx, ctx, Some(F(f)))), out) => f(input, Some(ctx), out),
            }}"#,
            ),
            Action::Push(ref s) => {
                let src = s.to_src();
                format!(
                    r#"match state_{dst}(input, Some({src}), ({f})(acc, token))? {{
                (None | Some((_, _, None)), _) => Err(Error::Unclosed {{ opened: index, delimiter: {src}, }}),
                (Some((idx, ctx, Some(F(f)))), out) => f(input, Some(ctx), out),
            }}"#,
                )
            }
            Action::Pop => match stack_symbol {
                Some(Some(s)) => {
                    format!("Ok((Some((index, {s}, Some(F(state_{dst})))), ({f})(acc, token)))")
                }
                _ => todo!(),
            },
        }
    }
}
