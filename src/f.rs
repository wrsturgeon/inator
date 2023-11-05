/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Function representations.

#![allow(clippy::module_name_repetitions)]

use core::{convert::identity as id, mem, ops};
use inator_automata::*;
use std::collections::BTreeSet;

/// One-argument function.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct F {
    /// Source-code representation of this function.
    pub src: String,
    /// Argument type.
    pub arg_t: String,
    /// Output type.
    pub output_t: String,
}

/// Two-argument function.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FF {
    /// Source-code representation of this function.
    pub src: String,
    /// Type of the first argument.
    pub lhs_t: String,
    /// Type of the second argument.
    pub rhs_t: String,
    /// Output type.
    pub output_t: String,
}

impl F {
    /// Internals of the `f!(...)` macro.
    #[inline]
    #[must_use]
    pub fn _from_macro<Arg: ToSrc, Output: ToSrc>(src: String, _: fn(Arg) -> Output) -> Self {
        Self {
            src,
            arg_t: Arg::src_type(),
            output_t: Output::src_type(),
        }
    }
}

impl FF {
    /// Internals of the `ff!(...)` macro.
    #[inline]
    #[must_use]
    pub fn _from_macro<Lhs: ToSrc, Rhs: ToSrc, Output: ToSrc>(
        src: String,
        _: fn(Lhs, Rhs) -> Output,
    ) -> Self {
        Self {
            src,
            lhs_t: Lhs::src_type(),
            rhs_t: Rhs::src_type(),
            output_t: Output::src_type(),
        }
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> ops::Shr<F> for Graph<I, S, C> {
    type Output = Self;
    #[inline]
    #[must_use]
    #[allow(clippy::panic, clippy::todo)]
    fn shr(mut self, rhs: F) -> Self::Output {
        let Ok(out_t) = self.output_type() else {
            panic!("Type inconsistency in the parser argument to `process`.")
        };
        if out_t.as_deref() != Some(&rhs.arg_t) {
            panic!(
                "Called `process` with a function that wants an input of type `{}`, \
            but the parser {}.",
                rhs.arg_t,
                out_t.map_or_else(|| "never returns".to_owned(), |t| format!("returns `{t}`"))
            );
        }
        let accepting_indices: BTreeSet<usize> = self
            .states
            .iter()
            .enumerate()
            .filter(|&(_, s)| s.non_accepting.is_empty())
            .map(|(i, _)| i)
            .collect();
        for state in &mut self.states {
            for curry in state.transitions.values_mut() {
                for transition in curry.values_mut() {
                    let to_accepting = transition
                        .dst
                        .view()
                        .map(|r| r.map_or_else(|tag| *unwrap!(self.tags.get(tag)), id))
                        .any(|i| accepting_indices.contains(&i));
                    if !to_accepting {
                        continue;
                    }
                    {
                        let old_out_t =
                            mem::replace(&mut transition.update.output_t, rhs.output_t.clone());
                        assert_eq!(
                            old_out_t, rhs.arg_t,
                            "Tried to apply a function to the output of a parser, but \
                        at least one path in the parser produced a mismatched type: \
                        the post-processing function wanted an input of type {}, but \
                        a path to an accepting state produced a value of type {old_out_t}",
                            rhs.arg_t,
                        );
                    }
                    let src = mem::take(&mut transition.update.src);
                    transition.update.src = format!("|tok, inp| ({})({src}(tok, inp))", rhs.src);
                }
            }
        }
        self
    }
}
