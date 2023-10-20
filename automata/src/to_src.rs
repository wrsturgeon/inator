/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Translate an automaton into Rust source code.

use crate::{
    Action, CurryInput, CurryStack, Graph, Input, Output, Range, RangeMap, Stack, State, Transition,
};
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

impl<I: Input, S: Stack, O: Output> Graph<I, S, O, usize> {
    /// Translate a value into Rust source code that reproduces it.
    #[inline]
    #[must_use]
    pub fn to_src(&self) -> String {
        let output_t = O::src_type();
        format!(
            "\
            type R<I> = Result<(F, {output_t}), TODO_ERROR_TYPE>;\r\n\
            type F<I> = fn(&mut I, Option<{}>, {output_t}) -> R<I>;\r\n\
            \r\n\
            #[inline]\r\n\
            pub fn parse<I: IntoIterator<Item = {}>>(input: I) -> Result<{output_t}, {}> {{\r\n\
                match state_{}(&mut input.into_iter(), None, <{output_t} as Default>::default())? {{\r\n\
                    (None, out) => Ok(out),\r\n\
                    (Some(..), _) => Err(POPPED_EMPTY_STACK),\r\n\
                }}\r\n\
            }}{}",
            S::src_type(),
            I::src_type(),
            "TODO_ERROR_TYPE",
            self.initial,
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

impl<I: Input, S: Stack, O: Output> State<I, S, O, usize> {
    /// Translate a value into Rust source code that reproduces it.
    #[inline]
    #[must_use]
    fn to_src(&self, i: usize) -> String {
        let output_t = O::src_type();
        format!(
            "\
            \r\n\
            \r\n\
            #[inline]\r\n\
            fn state_{i}<I: Iterator<Item = {}>>(input: &mut I, context: Option<{}>, acc: {output_t}) -> R {{\r\n\
                match input.next() {{\r\n\
                    None => {},\r\n\
                    Some(token) => {},\r\n\
                }}\r\n\
            }}",
            I::src_type(),
            S::src_type(),
            if self.accepting {
                "Ok((None, acc))"
            } else {
                "Err(TODO)"
            },
            self.transitions.to_src(),
        )
    }
}

impl<I: Input, S: Stack, O: Output> CurryStack<I, S, O, usize> {
    /// Translate a value into Rust source code that reproduces it.
    #[inline]
    #[must_use]
    #[allow(clippy::arithmetic_side_effects)] // <-- string concatenation with `+`
    fn to_src(&self) -> String {
        format!(
            "\
            match (&context, &token) {{{}\r\n
                _ => Err(TODO),\r\n
            }}",
            self.wildcard
                .iter()
                .map(|v| ("_".to_owned(), v))
                .chain(self.map_none.iter().map(|v| ("&None".to_owned(), v)))
                .chain(
                    self.map_some
                        .iter()
                        .map(|(k, v)| (format!("&Some({})", k.to_src()), v))
                )
                .fold(String::new(), |acc, (k, v)| acc + &v.to_src(&k)),
        )
    }
}

impl<I: Input, S: Stack, O: Output> CurryInput<I, S, O, usize> {
    /// Translate a value into Rust source code that reproduces it.
    #[inline]
    #[must_use]
    fn to_src(&self, stack_symbol: &str) -> String {
        match *self {
            Self::Wildcard(ref etc) => {
                format!("\r\n    ({stack_symbol}, _) => {},", etc.to_src(),)
            }
            Self::Scrutinize(ref etc) => etc.to_src(stack_symbol),
        }
    }
}

impl<I: Input, S: Stack, O: Output> RangeMap<I, S, O, usize> {
    /// Translate a value into Rust source code that reproduces it.
    #[inline]
    #[must_use]
    fn to_src(&self, stack_symbol: &str) -> String {
        self.entries
            .iter()
            .fold(String::new(), |acc, &(ref k, ref v)| {
                format!(
                    "{acc}\r\n    ({stack_symbol}, &({})) => {},",
                    k.to_src(),
                    v.to_src(),
                )
            })
    }
}

impl<I: Input, S: Stack, O: Output> Transition<I, S, O, usize> {
    /// Translate a value into Rust source code that reproduces it.
    #[inline]
    #[must_use]
    fn to_src(&self) -> String {
        let dst = self.dst;
        let f = self.update.src;
        match self.act {
            Action::Local => format!(
                "\
                match state_{dst}(input, context, ({f})(acc, token))? {{\r\n\
                    (None, acc) => if context.is_none() {{ Ok((None, acc)) }} else {{ Err(FINISHED_WITHOUT_EMPTYING_STACK) }},\r\n\
                    (Some(f), acc) => f(input, context, acc),\r\n\
                }}",
            ),
            Action::Push(ref s) => {
                format!(
                    "\
                    match state_{dst}(input, Some({}), ({f})(acc, token))? {{\r\n\
                        (None, acc) => if context.is_none() {{ Ok((None, acc)) }} else {{ Err(FINISHED_WITHOUT_EMPTYING_STACK) }},\r\n\
                        (Some(f), acc) => f(input, context, acc);\r\n\
                    }}",
                    s.to_src(),
                )
            }
            Action::Pop => format!("return Ok((Some(state_{dst}), ({f})(acc, token)))"),
        }
    }
}
