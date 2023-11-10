/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Translate an automaton into Rust source code.

use crate::{
    Ctrl, Curry, Deterministic, Graph, IllFormed, Input, Range, RangeMap, State, Transition,
    Update, FF,
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

impl ToSrc for u16 {
    #[inline]
    #[must_use]
    fn to_src(&self) -> String {
        format!("{self}")
    }
    #[inline]
    #[must_use]
    fn src_type() -> String {
        "u16".to_owned()
    }
}

impl ToSrc for u32 {
    #[inline]
    #[must_use]
    fn to_src(&self) -> String {
        format!("{self}")
    }
    #[inline]
    #[must_use]
    fn src_type() -> String {
        "u32".to_owned()
    }
}

impl ToSrc for u64 {
    #[inline]
    #[must_use]
    fn to_src(&self) -> String {
        format!("{self}")
    }
    #[inline]
    #[must_use]
    fn src_type() -> String {
        "u64".to_owned()
    }
}

impl ToSrc for u128 {
    #[inline]
    #[must_use]
    fn to_src(&self) -> String {
        format!("{self}")
    }
    #[inline]
    #[must_use]
    fn src_type() -> String {
        "u128".to_owned()
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
        let token_t = I::src_type();
        let output_t = self.output_type()?.unwrap_or("core::convert::Infallible");
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
        token: {token_t},
    }},
    /// Token that would have closed a delimiter, but the delimiter wasn't open.
    Unopened {{
        /// What was actually open, if anything, and the index of the token that opened it.
        what_was_open: Option<(&'static str, usize)>,
        /// Index of the token that caused this error.
        index: usize,
    }},
    /// After parsing all input, a delimiter remains open (e.g. "(a, b, c").
    Unclosed {{
        /// Region (user-defined name) that was not closed. Sensible to be e.g. "parentheses" for `(...)`.
        region: &'static str,
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
pub fn parse<I: IntoIterator<Item = {token_t}>>(input: I) -> Result<{output_t}, Error> {{
    state_{}(&mut input.into_iter().enumerate(), (), None)
}}{}
"#,
            self.initial,
            self.states
                .iter()
                .enumerate()
                .try_fold(String::new(), |acc, (i, s)| Ok(
                    acc + &s.to_src(i, &self.states, &self.tags)?
                ))?,
        ))
    }
}

impl<I: Input> State<I, usize> {
    /// Translate a value into Rust source code that reproduces it.
    #[inline]
    fn to_src(
        &self,
        i: usize,
        all_states: &[Self],
        all_tags: &BTreeMap<String, usize>,
    ) -> Result<String, IllFormed<I, usize>> {
        let input_t = self
            .input_type(all_states, all_tags)?
            .unwrap_or("core::convert::Infallible");
        let token_t = I::src_type();
        let on_some = self.transitions.to_src();
        let on_none = self.non_accepting.first().map_or_else(
            || {
                "stack_top.map_or(
            Ok(acc),
            |(region, opened)| Err(Error::Unclosed { region, opened }),
        )"
                .to_owned()
            },
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
            },
        );
        Ok(format!(
            r#"


#[inline]
fn state_{i}<I: Iterator<Item = (usize, {token_t})>>(input: &mut I, acc: {input_t}, stack_top: Option<(&'static str, usize)>) -> Result<{input_t}, Error> {{
    match input.next() {{
        None => {on_none},
        Some((index, token)) => match token {{{on_some}
            _ => Err(Error::Absurd {{ index, token }}),
        }},
    }}
}}"#,
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
        self.0.iter().fold(String::new(), |acc, (k, v)| {
            format!(
                r#"{acc}
            {} => {},"#,
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
    fn to_src(&self) -> String {
        match *self {
            Self::Lateral {
                dst,
                update: Update { ref src, .. },
            } => format!("state_{dst}(input, ({src})(acc, token), stack_top)"),
            Self::Call {
                region,
                detour,
                dst,
                combine: FF { ref src, .. },
            } => format!(
                r#"{{
                let detour = state_{detour}(input, (), Some(({}, index)))?;
                let postprocessed = ({src})(acc, detour);
                state_{dst}(input, postprocessed, stack_top)
            }}"#,
                region.to_src(),
            ),
            Self::Return { region } => {
                format!(
                    "match stack_top {{
                Some((region, _)) if region == {} => Ok(acc),
                _ => Err(Error::Unopened {{ what_was_open: stack_top, index }})
            }}",
                    region.to_src(),
                )
            }
        }
    }
}

impl<I: Input, C: Ctrl<I>> ToSrc for Graph<I, C> {
    #[inline]
    fn to_src(&self) -> String {
        format!(
            "{} {{ states: {}, initial: {} }}",
            Self::src_type(),
            self.states.to_src(),
            self.initial.to_src(),
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
        format!("State::<{}, BTreeSet<usize>>", I::src_type(),)
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
        format!("Curry::<{}, BTreeSet<usize>>", I::src_type(),)
    }
}

impl<I: Input, C: Ctrl<I>> ToSrc for RangeMap<I, C> {
    #[inline]
    fn to_src(&self) -> String {
        format!("RangeMap({})", self.0.to_src())
    }
    #[inline]
    fn src_type() -> String {
        format!("RangeMap::<{}, BTreeSet<usize>>", I::src_type(),)
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
            Self::Call {
                region,
                ref detour,
                ref dst,
                ref combine,
            } => format!(
                "Transition::Call {{ region: {}, detour: {}, dst: {}, combine: {} }}",
                region.to_src(),
                detour.to_src(),
                dst.to_src(),
                combine.to_src(),
            ),
            Self::Return { region } => {
                format!("Transition::Return {{ region: {} }}", region.to_src())
            }
        }
    }
    #[inline]
    fn src_type() -> String {
        format!("Transition::<{}, BTreeSet<usize>>", I::src_type(),)
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

impl ToSrc for FF {
    #[inline]
    fn to_src(&self) -> String {
        format!("ff!({})", self.src)
    }
    #[inline]
    fn src_type() -> String {
        "FF".to_owned()
    }
}
