/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Translate an automaton into Rust source code.

use crate::{
    Ctrl, Curry, Deterministic, Graph, IllFormed, Input, Range, RangeMap, State, Transition, Update,
};
use core::ops::Bound;
use std::collections::{BTreeMap, BTreeSet};

/// Translate a value into Rust source code that reproduces it.
pub trait ToSrc {
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
        // format!("{self}")
        format!("b'{}'", self.escape_ascii())
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
            "[{}, {}{}].into_iter().collect()",
            fst.to_src(),
            snd.to_src(),
            iter.fold(String::new(), |acc, x| format!("{acc}, {}", x.to_src())),
        )
    }
    #[inline]
    #[must_use]
    fn src_type() -> String {
        format!("std::collections::BTreeSet::<{}>", T::src_type())
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
            |x| format!("Ok({})", x.to_src()),
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
        if self.is_empty() {
            "String::new()".to_owned()
        } else {
            format!("\"{}\".to_owned()", self.escape_default())
        }
    }
    #[inline]
    #[must_use]
    fn src_type() -> String {
        "String".to_owned()
    }
}

impl ToSrc for &str {
    #[inline]
    #[must_use]
    fn to_src(&self) -> String {
        format!("\"{}\"", self.escape_default())
    }
    #[inline]
    #[must_use]
    fn src_type() -> String {
        "&'static str".to_owned()
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

impl<I: Input> Deterministic<I> {
    /// Translate a value into Rust source code that reproduces it.
    /// # Errors
    /// If this automaton is ill-formed.
    #[inline]
    #[allow(clippy::arithmetic_side_effects)] // <-- String concatenation with `+`
    pub fn to_src(&self) -> Result<String, IllFormed<I, usize>> {
        let input_t = I::src_type();
        let output_t = self.output_type()?.unwrap_or_else(|| {
            /* "core::convert::Infallible" */
            "()".to_owned()
        });
        Ok(format!(
            r#"//! Automatically generated with [inator](https://crates.io/crates/inator).

#![allow(dead_code, unused_variables)]

/// Descriptive parsing error.
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
    }},
    /// After parsing all input, a delimiter remains open (e.g. "(a, b, c").
    Unclosed {{
        /// Index at which the delimiter was opened (e.g., for parentheses, the index of the relevant '(').
        opened: usize,
    }},
    /// Ended on a user-defined non-accepting state.
    UserDefined {{
        /// User-defined error message.
        messages: &'static [&'static str],
    }},
}}

type R<I> = Result<(Option<(usize, Option<F<I>>)>, {output_t}), Error>;

#[repr(transparent)]
struct F<I>(fn(&mut I, {output_t}) -> R<I>);

#[inline]
pub fn parse<I: IntoIterator<Item = {input_t}>>(input: I) -> Result<{output_t}, Error> {{
    match state_{}(
        &mut input.into_iter().enumerate(),
        None,
        Default::default(),
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
                .try_fold(String::new(), |acc, (i, s)| Ok(acc + &s.to_src(i)?))?,
        ))
    }
}

impl<I: Input> State<I, usize> {
    /// Translate a value into Rust source code that reproduces it.
    #[inline]
    fn to_src(&self, i: usize) -> Result<String, IllFormed<I, usize>> {
        let input_t = self.input_type()?.unwrap_or_else(|| {
            /* "core::convert::Infallible" */
            "()".to_owned()
        });
        Ok(format!(
            r#"


#[inline]
fn state_{i}<I: Iterator<Item = (usize, {})>>(input: &mut I, acc: {input_t}) -> R<I> {{
    match input.next() {{
        None => {},
        Some((index, token)) => {},
    }}
}}"#,
            I::src_type(),
            self.non_accepting.first().map_or_else(
                || "Ok((None, acc))".to_owned(),
                |fst| {
                    self.non_accepting
                        .range((Bound::Excluded(fst.clone()), Bound::Unbounded))
                        .fold(
                            format!(
                                "Err(Error::UserDefined {{ messages: &[{}",
                                fst.as_str().to_src(),
                            ),
                            |acc, msg| format!("{acc}, {}", msg.as_str().to_src()),
                        )
                        + "] })"
                }
            ),
            self.transitions.to_src(),
        ))
    }
}

impl<I: Input> Curry<I, usize> {
    /// Translate a value into Rust source code that reproduces it.
    #[inline]
    #[must_use]
    fn to_src(&self) -> String {
        match *self {
            Self::Wildcard(ref etc) => format!(
                r#"
            _ => {},"#,
                etc.to_src(),
            ),
            Self::Scrutinize(ref etc) => etc.to_src(),
        }
    }
}

impl<I: Input> RangeMap<I, usize> {
    /// Translate a value into Rust source code that reproduces it.
    #[inline]
    #[must_use]
    fn to_src(&self) -> String {
        self.entries.iter().fold(String::new(), |acc, (k, v)| {
            format!(
                r#"{acc}
            &({}) => {},"#,
                k.to_src(),
                v.to_src(),
            )
        })
    }
}

impl<I: Input> Transition<I, usize> {
    /// Translate a value into Rust source code that reproduces it.
    #[inline]
    #[must_use]
    #[allow(clippy::todo)] // TODO: what the fuck does the last case mean?
    fn to_src(&self) -> String {
        match self {
            Self::Lateral { dst, update } => format!(
                r#"match state_{dst}(input, context, ({})(acc, token))? {{
                (None, _) => todo!(),
                (done @ Some((_, _, None)), acc) => Ok((done, acc)),
                (Some((idx, ctx, Some(F(f)))), out) => f(input, Some(ctx), out),
            }}"#,
                update.src,
            ),
            _ => todo!(),
        }
    }
}

impl<I: Input, C: Ctrl<I>> ToSrc for Graph<I, C> {
    #[inline]
    fn to_src(&self) -> String {
        format!(
            "Nondeterministic {{ states: {}, initial: {}, tags: {} }}",
            self.states.to_src(),
            self.initial.to_src(),
            self.tags.to_src(),
        )
    }
    #[inline]
    fn src_type() -> String {
        format!("Nondeterministic::<{}>", I::src_type())
    }
}

impl<T: ToSrc> ToSrc for Vec<T> {
    #[inline]
    fn to_src(&self) -> String {
        self.first().map_or_else(
            || "vec![]".to_owned(),
            |fst| {
                format!(
                    "vec![{}{}]",
                    fst.to_src(),
                    get!(self, 1..)
                        .iter()
                        .fold(String::new(), |acc, x| format!("{acc}, {}", x.to_src()))
                )
            },
        )
    }
    #[inline]
    fn src_type() -> String {
        format!("Vce::<{}>", T::src_type())
    }
}

impl<I: Input, C: Ctrl<I>> ToSrc for State<I, C> {
    #[inline]
    fn to_src(&self) -> String {
        format!(
            "State {{ transitions: {}, non_accepting: {} }}",
            self.transitions.to_src(),
            self.non_accepting.to_src(),
        )
    }
    #[inline]
    fn src_type() -> String {
        format!(
            "State::<{}, BTreeSet<Result<usize, String>>>",
            I::src_type(),
        )
    }
}

impl<I: Input, C: Ctrl<I>> ToSrc for Curry<I, C> {
    #[inline]
    fn to_src(&self) -> String {
        match *self {
            Self::Wildcard(ref w) => format!("Curry::Wildcard({})", w.to_src()),
            Self::Scrutinize(ref s) => format!("Curry::Scrutinize({})", s.to_src()),
        }
    }
    #[inline]
    fn src_type() -> String {
        format!(
            "Curry::<{}, BTreeSet<Result<usize, String>>>",
            I::src_type(),
        )
    }
}

impl<I: Input, C: Ctrl<I>> ToSrc for RangeMap<I, C> {
    #[inline]
    fn to_src(&self) -> String {
        format!("RangeMap {{ entries: {} }}", self.entries.to_src())
    }
    #[inline]
    fn src_type() -> String {
        format!(
            "RangeMap::<{}, BTreeSet<Result<usize, String>>>",
            I::src_type(),
        )
    }
}

impl<K: Clone + Ord + ToSrc, V: Clone + ToSrc> ToSrc for BTreeMap<K, V> {
    #[inline]
    fn to_src(&self) -> String {
        match self.len() {
            0 => "BTreeMap::new()".to_owned(),
            1 => format!("iter::once({}).collect()", {
                let (k, v) = unwrap!(self.first_key_value());
                (k.clone(), v.clone()).to_src()
            }),
            _ => format!(
                "{}.into_iter().collect()",
                self.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<Vec<_>>()
                    .to_src()
            ),
        }
    }
    #[inline]
    fn src_type() -> String {
        format!("BTreeMap::<{}, {}>", K::src_type(), V::src_type())
    }
}

impl<A: ToSrc, B: ToSrc> ToSrc for (A, B) {
    #[inline]
    fn to_src(&self) -> String {
        format!("({}, {})", self.0.to_src(), self.1.to_src())
    }
    #[inline]
    fn src_type() -> String {
        format!("({}, {})", A::src_type(), B::src_type())
    }
}

impl<I: Input, C: Ctrl<I>> ToSrc for Transition<I, C> {
    #[inline]
    fn to_src(&self) -> String {
        match *self {
            Self::Lateral {
                ref dst,
                ref update,
            } => format!(
                "Transition::Lateral {{ dst: {}, update: {} }}",
                dst.to_src(),
                update.to_src(),
            ),
            Self::Call {} | Self::Return {} => todo!(),
        }
    }
    #[inline]
    fn src_type() -> String {
        format!(
            "Transition::<{}, BTreeSet<Result<usize, String>>>",
            I::src_type(),
        )
    }
}

impl<I: Input> ToSrc for Update<I> {
    #[inline]
    fn to_src(&self) -> String {
        format!("update!({})", self.src.to_src())
    }
    #[inline]
    fn src_type() -> String {
        format!("Update::<{}>", I::src_type())
    }
}
