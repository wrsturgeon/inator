/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! State, i.e. a node in an automaton graph.

use crate::{Ctrl, CurryStack, IllFormed, Input, Merge, Stack};
use core::{cmp, convert::identity as id};
use std::collections::{BTreeMap, BTreeSet};

/// State, i.e. a node in an automaton graph.
#[allow(clippy::exhaustive_structs)]
#[derive(Debug)]
pub struct State<I: Input, C: Ctrl<I>> {
    /// Map from input tokens to actions.
    pub transitions: Curry<I, C>,
    /// If input ends while in this state, should we accept?
    pub non_accepting: BTreeSet<String>,
}

impl<I: Input, C: Ctrl<I>> State<I, C> {
    /// Compute the input type of any run that reaches this state.
    /// # Errors
    /// If multiple transitions expect different types.
    #[inline]
    pub fn input_type(&self) -> Result<Option<&str>, IllFormed<I, C>> {
        self.transitions
            .values()
            .try_fold(None, |acc: Option<&str>, t| {
                let in_t = t.input_type();
                acc.map_or(Ok(in_t), |other| match in_t {
                    None => Ok(Some(other)),
                    Some(ty) if ty == other => Ok(Some(other)),
                    Some(ty) => Err(IllFormed::TypeMismatch(other.to_owned(), ty.to_owned())),
                })
            })
            .map_or_else(
                |(a, b)| {
                    if a == b {
                        Ok(Some(a))
                    } else {
                        Err(IllFormed::TypeMismatch(a, b))
                    }
                },
                Ok,
            )
        })?;

        // Next, look at all states that transition into this one.
        for state in all_states {
            for curry in state.transitions.values() {
                for transition in curry.values() {
                    let to_here = transition
                        .dst
                        .view()
                        .map(|r| {
                            get!(
                                all_states,
                                r.map_or_else(|tag| *unwrap!(all_tags.get(tag)), id)
                            )
                        })
                        .any(|s| s == self);
                    if !to_here {
                        continue;
                    }
                    best_guess = best_guess
                        .merge(Some(transition.update.output_t.clone()))
                        .map_or_else(
                            |(a, b)| {
                                if a == b {
                                    Ok(Some(a))
                                } else {
                                    Err(IllFormed::TypeMismatch(a, b))
                                }
                            },
                            Ok,
                        )?;
                }
            }
        }
        Ok(best_guess)
    }
}

impl<I: Input> State<I, usize> {
    /// Convert the control parameter from `usize` to anything else.
    #[inline]
    #[must_use]
    pub fn convert_ctrl<C: Ctrl<I>>(self) -> State<I, C> {
        State {
            transitions: self.transitions.convert_ctrl(),
            non_accepting: self.non_accepting,
        }
    }
}

impl<I: Input, C: Ctrl<I>> Clone for State<I, C> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            transitions: self.transitions.clone(),
            non_accepting: self.non_accepting.clone(),
        }
    }
}

impl<I: Input, C: Ctrl<I>> Eq for State<I, C> {}

impl<I: Input, C: Ctrl<I>> PartialEq for State<I, C> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.transitions == other.transitions && self.non_accepting == other.non_accepting
    }
}

impl<I: Input, C: Ctrl<I>> Ord for State<I, C> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.transitions
            .cmp(&other.transitions)
            .then_with(|| self.non_accepting.cmp(&other.non_accepting))
    }
}

impl<I: Input, C: Ctrl<I>> PartialOrd for State<I, C> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}
