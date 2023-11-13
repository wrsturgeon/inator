/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Read the next input symbol and decide an action.

use crate::{Ctrl, IllFormed, Input, Range, RangeMap, Transition};
use core::{cmp, iter};
use std::collections::BTreeMap;

/// Read the next input symbol and decide an action.
#[allow(clippy::exhaustive_enums)]
#[derive(Debug)]
pub enum Curry<I: Input, C: Ctrl<I>> {
    /// Throw away the input (without looking at it) and do this.
    Wildcard(Transition<I, C>),
    /// Map specific ranges of inputs to actions.
    Scrutinize {
        /// Specific ranges to route to various transitions.
        filter: RangeMap<I, C>,
        /// If no ranges match, take this transition (if any; otherwise, fail).
        fallback: Option<Transition<I, C>>,
    },
}

impl<I: Input, C: Ctrl<I>> Clone for Curry<I, C> {
    #[inline]
    fn clone(&self) -> Self {
        match *self {
            Self::Wildcard(ref etc) => Self::Wildcard(etc.clone()),
            Self::Scrutinize {
                ref filter,
                ref fallback,
            } => Self::Scrutinize {
                filter: filter.clone(),
                fallback: fallback.clone(),
            },
        }
    }
}

impl<I: Input, C: Ctrl<I>> Eq for Curry<I, C> {}

impl<I: Input, C: Ctrl<I>> PartialEq for Curry<I, C> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&Self::Wildcard(ref a), &Self::Wildcard(ref b)) => a == b,
            (
                &Self::Scrutinize {
                    filter: ref l_filter,
                    fallback: ref l_fallback,
                },
                &Self::Scrutinize {
                    filter: ref r_filter,
                    fallback: ref r_fallback,
                },
            ) => (l_filter, l_fallback) == (r_filter, r_fallback),
            (&Self::Wildcard(..), &Self::Scrutinize { .. })
            | (&Self::Scrutinize { .. }, &Self::Wildcard(..)) => false, // unfortunately no general way to tell if a range covers a whole type
        }
    }
}

impl<I: Input, C: Ctrl<I>> Ord for Curry<I, C> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match (self, other) {
            (&Self::Wildcard(ref a), &Self::Wildcard(ref b)) => a.cmp(b),
            (&Self::Wildcard(..), &Self::Scrutinize { .. }) => cmp::Ordering::Less,
            (&Self::Scrutinize { .. }, &Self::Wildcard(..)) => cmp::Ordering::Greater,
            (
                &Self::Scrutinize {
                    filter: ref l_filter,
                    fallback: ref l_fallback,
                },
                &Self::Scrutinize {
                    filter: ref r_filter,
                    fallback: ref r_fallback,
                },
            ) => (l_filter, l_fallback).cmp(&(r_filter, r_fallback)),
        }
    }
}

impl<I: Input, C: Ctrl<I>> PartialOrd for Curry<I, C> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<I: Input, C: Ctrl<I>> Curry<I, C> {
    /// Look up a transition based on an input token.
    /// # Errors
    /// If multiple ranges fit an argument.
    #[inline]
    #[allow(clippy::type_complexity)]
    pub fn get(&self, key: &I) -> Result<Option<&Transition<I, C>>, IllFormed<I, C>> {
        match *self {
            Self::Wildcard(ref transition) => Ok(Some(transition)),
            Self::Scrutinize {
                ref filter,
                ref fallback,
            } => Ok(filter.get(key)?.or(fallback.as_ref())),
        }
    }

    /// Assert that this map has no keys in common with another.
    /// # Errors
    /// If there are keys in common, don't panic: instead, return them.
    /// Here's the meaning of an error:
    /// - `None`: Conflict on a fallback.
    /// - `Some(None, ..)`: Conflict on literally anything. Means both are wildcards.
    /// - `Some(Some(range), ..)`: Conflict on at least this range of values,
    ///   which is an intersection of two offending ranges.
    #[inline]
    #[allow(clippy::result_large_err, clippy::type_complexity)]
    pub fn disjoint(
        &self,
        other: &Self,
    ) -> Result<(), Option<(Option<Range<I>>, Transition<I, C>, Transition<I, C>)>> {
        match (self, other) {
            (&Self::Wildcard(ref a), &Self::Wildcard(ref b)) => {
                Err(Some((None, a.clone(), b.clone())))
            }
            (
                &Self::Wildcard(ref w),
                &Self::Scrutinize {
                    ref filter,
                    ref fallback,
                },
            )
            | (
                &Self::Scrutinize {
                    ref filter,
                    ref fallback,
                },
                &Self::Wildcard(ref w),
            ) => filter.0.first_key_value().map_or_else(
                || fallback.as_ref().map_or(Ok(()), |_| Err(None)),
                |(k, v)| Err(Some((Some(k.clone()), w.clone(), v.clone()))),
            ),
            (
                &Self::Scrutinize {
                    filter: ref l_filter,
                    fallback: ref l_fallback,
                },
                &Self::Scrutinize {
                    filter: ref r_filter,
                    fallback: ref r_fallback,
                },
            ) => l_filter.disjoint(r_filter).map_or_else(
                |(intersection, lv, rv)| Err(Some((Some(intersection), lv, rv))),
                |()| {
                    if l_fallback.is_some() && r_fallback.is_some() {
                        Err(None)
                    } else {
                        Ok(())
                    }
                },
            ),
        }
    }

    /// All values in this collection, without their associated keys.
    #[inline]
    pub fn values(&self) -> Box<dyn '_ + Iterator<Item = &Transition<I, C>>> {
        match *self {
            Self::Wildcard(ref etc) => Box::new(iter::once(etc)),
            Self::Scrutinize {
                ref filter,
                ref fallback,
            } => Box::new(filter.values().chain(fallback)),
        }
    }

    /// Remove an entry by key.
    /// # Panics
    /// If we ask to remove a wildcard but it's a specific value, or vice-versa.
    #[inline]
    pub fn remove(&mut self, key: Option<Range<I>>) {
        match *self {
            Self::Wildcard(..) => {
                // assert!(
                //     key.is_none(),
                //     "Asked to remove a specific value \
                //     but the map took a wildcard",
                // );
                *self = Self::Scrutinize {
                    filter: RangeMap(BTreeMap::new()),
                    fallback: None,
                };
            }
            Self::Scrutinize {
                ref mut filter,
                ref fallback,
            } => {
                filter.remove(&key.expect(
                    "Asked to remove a wildcard \
                    but the map took a specific value",
                ));
                assert!(
                    fallback.is_none(),
                    "Asked to remove a value but the map has a fallback",
                );
            }
        };
    }

    /// All values in this collection, without their associated keys.
    #[inline]
    pub fn values_mut(&mut self) -> Box<dyn '_ + Iterator<Item = &mut Transition<I, C>>> {
        match *self {
            Self::Wildcard(ref mut etc) => Box::new(iter::once(etc)),
            Self::Scrutinize {
                ref mut filter,
                ref mut fallback,
            } => Box::new(filter.values_mut().chain(fallback)),
        }
    }

    /// Check if this parser ever could, at any point, involve a fallback transition.
    #[inline]
    #[must_use]
    pub const fn involves_any_fallback(&self) -> bool {
        matches!(
            *self,
            Self::Scrutinize {
                fallback: Some(_),
                ..
            }
        )
    }
}

impl<I: Input> Curry<I, usize> {
    /// Convert the control parameter from `usize` to anything else.
    #[inline]
    #[must_use]
    pub fn convert_ctrl<C: Ctrl<I>>(self) -> Curry<I, C> {
        match self {
            Curry::Wildcard(w) => Curry::Wildcard(w.convert_ctrl()),
            Curry::Scrutinize { filter, fallback } => Curry::Scrutinize {
                filter: filter.convert_ctrl(),
                fallback: fallback.map(Transition::convert_ctrl),
            },
        }
    }
}
