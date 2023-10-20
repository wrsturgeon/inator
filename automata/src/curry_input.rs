/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Read the next input symbol and decide an action.

use crate::{Ctrl, IllFormed, Input, Output, Range, RangeMap, Stack, Transition};

/// Read the next input symbol and decide an action.
#[allow(clippy::exhaustive_enums)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurryInput<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> {
    /// Throw away the input (without looking at it) and do this.
    Wildcard(Transition<I, S, O, C>),
    /// Map specific ranges of inputs to actions.
    Scrutinize(RangeMap<I, S, O, C>),
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> CurryInput<I, S, O, C> {
    /// Look up a transition based on an input token.
    /// # Errors
    /// If multiple ranges fit an argument.
    #[inline]
    #[allow(clippy::type_complexity)]
    pub fn get(&self, key: &I) -> Result<Option<&Transition<I, S, O, C>>, IllFormed<I, S, O, C>> {
        match *self {
            Self::Wildcard(ref transition) => Ok(Some(transition)),
            Self::Scrutinize(ref range_map) => range_map.get(key),
        }
    }

    /// Assert that this map has no keys in common with another.
    /// # Errors
    /// If there are keys in common, don't panic: instead, return them.
    /// Here's the format of what it returns:
    /// - `None`: Conflict on literally anything. Means both are wildcards.
    /// - `Some(range)`: Conflict on at least this range of values,
    ///   which is an intersection of two offending ranges.
    #[inline]
    #[allow(clippy::type_complexity)]
    pub fn disjoint(
        &self,
        other: &Self,
    ) -> Result<
        (),
        (
            Option<Range<I>>,
            Transition<I, S, O, C>,
            Transition<I, S, O, C>,
        ),
    > {
        match (self, other) {
            (&Self::Wildcard(ref a), &Self::Wildcard(ref b)) => {
                if a == b {
                    Ok(())
                } else {
                    Err((None, a.clone(), b.clone()))
                }
            }
            (&Self::Wildcard(ref w), &Self::Scrutinize(ref s))
            | (&Self::Scrutinize(ref s), &Self::Wildcard(ref w)) => {
                s.entries.iter().try_fold((), |(), &(ref k, ref v)| {
                    if w == v {
                        Ok(())
                    } else {
                        Err((Some(k.clone()), w.clone(), v.clone()))
                    }
                })
            }
            (&Self::Scrutinize(ref a), &Self::Scrutinize(ref b)) => a
                .disjoint(b)
                .map_err(|(intersection, lv, rv)| (Some(intersection), lv, rv)),
        }
    }
}
