/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Translate an automaton into Rust source code.

use crate::{Ctrl, CurryStack, Graph, Input, Output, Stack, State};
use core::iter;
use std::collections::BTreeSet;

/// Translate a value into Rust source code that reproduces it.
pub trait ToSrc {
    /// Translate a value into Rust source code that reproduces it.
    #[must_use]
    fn to_src(&self) -> String;
    /// Translate a type into Rust source code that reproduces it.
    #[must_use]
    fn src_type() -> String;
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
        match *self {
            None => "None".to_owned(),
            Some(ref x) => format!("Some({})", x.to_src()),
        }
    }
    #[inline]
    #[must_use]
    fn src_type() -> String {
        format!("Option::<{}>", T::src_type())
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Graph<I, S, O, C> {
    #[inline]
    #[must_use]
    fn to_src(&self) -> String {
        format!(
            "#[inline]
            pub fn parse<I: IntoIterator<Item = {}>>(input: I) -> Result<{}, {}> {{
                state_{}(&mut input.into_iter(), None)
            }}{}",
            I::src_type(),
            O::src_type(),
            "TODO_ERROR_TYPE",
            self.initial.to_src(),
            self.states
                .iter()
                .enumerate()
                .fold(String::new(), |acc, (i, s)| format!(
                    "{acc}\r\n\r\n{}",
                    s.to_src(i),
                )),
        )
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> State<I, S, O, C> {
    #[inline]
    #[must_use]
    fn to_src(&self, i: usize) -> String {
        format!(
            "#[inline]
            fn state_{i}<I: Iterator<Item = {}>>(input: &mut I, context: Option<{}>) -> Result<{}, {}> {{
                match input.next() {{
                    None => {},
                    Some(token) => {},
                }}
            }}",
            I::src_type(),
            S::src_type(),
            O::src_type(),
            "TODO_ERROR_TYPE",
            if self.accepting {
                "Ok(acc)"
            } else {
                "Err(TODO)"
            },
            self.transitions.to_src(),
        )
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> CurryStack<I, S, O, C> {
    #[inline]
    #[must_use]
    fn to_src(&self) -> String {
        format!(
            "match (context, token) {{{}
            }}",
            self.wildcard
                .iter()
                .map(|v| ("_".to_owned(), v))
                .chain(self.map_none.iter().map(|v| ("None".to_owned(), v)))
                .chain(
                    self.map_some
                        .iter()
                        .map(|(k, v)| (format!("Some({})", k.to_src()), v))
                )
                .fold(String::new(), |(k, v)| v.to_src(k)),
        )
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> CurryInput<I, S, O, C> {
    #[inline]
    #[must_use]
    fn to_src(&self, stack_symbol: String) -> String {
        match *self {
            Self::Wildcard(etc) => {
                format!(
                    "
                        ({stack_symbol}, _) => {},",
                    etc.to_src(),
                )
            }
            Self::Scrutinize(etc) => etc.to_src(stack_symbol),
        }
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> RangeMap<I, S, O, C> {
    #[inline]
    #[must_use]
    fn to_src(&self, stack_symbol: String) -> String {
        self.entries
            .iter()
            .fold(String::new(), |acc, &(ref k, ref v)| {
                format!(
                    "
                        ({stack_symbol}, {}) => {},",
                    k.to_src(),
                    v.to_src(),
                )
            })
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Transition<I, S, O, C> {
    #[inline]
    #[must_use]
    fn to_src(&self, stack_symbol: String) -> String {
        match self.act {
            Action::Local => todo!(),
            Action::Push(ref s) => todo!(),
            Action::Pop => todo!(),
        }
    }
}
