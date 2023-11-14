/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! `QuickCheck` implementations for various types.

use crate::{Ctrl, Curry, Graph, Input, Range, RangeMap, State, Transition, Update, FF};
use core::{iter, num::NonZeroUsize};
use quickcheck::{Arbitrary, Gen};
use std::collections::{BTreeMap, BTreeSet};

/// Sample a value uniformly below the maximum size allowed by a generator.
#[inline]
#[allow(clippy::arithmetic_side_effects)]
fn within_size(g: &mut Gen) -> usize {
    usize::arbitrary(g) % NonZeroUsize::new(g.size()).expect("Zero-sized QuickCheck generator")
}

impl<C: Arbitrary + Ctrl<u8>> Arbitrary for Graph<u8, C> {
    #[inline]
    #[allow(clippy::arithmetic_side_effects)]
    fn arbitrary(g: &mut Gen) -> Self {
        'restart: loop {
            let size = within_size(g);
            let Some(nz) = NonZeroUsize::new(size) else {
                continue 'restart;
            };
            let initial = C::arbitrary_given(nz, g);
            let mut states: Vec<_> = (0..size).map(|_| State::arbitrary_given(nz, g)).collect();
            'sort_again: loop {
                states.sort_unstable();
                states.dedup();
                let Some(nz_post) = NonZeroUsize::new(states.len()) else {
                    continue 'restart;
                };
                states = states
                    .into_iter()
                    .map(|s| s.map_indices(|i| i % nz_post))
                    .collect();
                // Check if `states` is still sorted
                for i in 1..states.len() {
                    if get!(states, i.overflowing_sub(1).0) >= get!(states, i) {
                        continue 'sort_again;
                    }
                }
                return Self {
                    states,
                    initial: initial.map_indices(|i| i % nz_post),
                };
            }
        }
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.states.clone(), self.initial.clone())
                .shrink()
                .filter_map(|(states, initial)| {
                    let s = Self { states, initial };
                    (s.check() == Ok(())).then_some(s)
                }),
        )
    }
}

impl<I: Arbitrary + Input> Arbitrary for Range<I> {
    #[inline]
    fn arbitrary(g: &mut Gen) -> Self {
        let (a, b) = <(I, I)>::arbitrary(g);
        if a <= b {
            Self { first: a, last: b }
        } else {
            Self { first: b, last: a }
        }
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.first.clone(), self.last.clone())
                .shrink()
                .map(|(a, b)| {
                    if a <= b {
                        Self { first: a, last: b }
                    } else {
                        Self { first: b, last: a }
                    }
                }),
        )
    }
}

impl<I: 'static + Input> Arbitrary for Update<I> {
    #[inline(always)]
    fn arbitrary(_: &mut Gen) -> Self {
        update!(|(), _| {})
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(iter::empty())
    }
}

/// Implement only `shrink`; let `arbitrary` panic.
/// Use when a value requires knowledge of the number of states in an automaton.
macro_rules! shrink_only {
    (|$self:ident: &$t:ident| $body:expr) => {
        impl<C: Arbitrary + Ctrl<u8>> Arbitrary for $t<u8, C> {
            #[inline(always)]
            fn arbitrary(_: &mut Gen) -> Self {
                never!()
            }
            #[inline]
            fn shrink(&$self) -> Box<dyn Iterator<Item = Self>> {
                $body
            }
        }
    };
}

shrink_only!(|self: &State| Box::new(
    (self.transitions.clone(), self.non_accepting.clone(),)
        .shrink()
        .map(|(transitions, non_accepting)| Self {
            transitions,
            non_accepting,
        })
));

shrink_only!(|self: &RangeMap| Box::new(self.0.shrink().map(Self)));

shrink_only!(|self: &Curry| match *self {
    Self::Wildcard(ref etc) => Box::new(etc.shrink().map(Self::Wildcard)),
    #[allow(clippy::shadow_unrelated)]
    Self::Scrutinize {
        ref filter,
        ref fallback,
    } => Box::new(
        filter
            .0
            .first_key_value()
            .map(|(_, transition)| Self::Wildcard(transition.clone()))
            .into_iter()
            .chain(
                (filter.clone(), fallback.clone())
                    .shrink()
                    .map(|(filter, fallback)| Self::Scrutinize { filter, fallback })
            )
    ),
});

shrink_only!(|self: &Transition| {
    #[allow(clippy::shadow_unrelated, unreachable_code, unused_variables)]
    match *self {
        Self::Return { .. } => Box::new(iter::empty()),
        Self::Lateral {
            ref dst,
            ref update,
        } => Box::new(
            iter::once(Self::Return { region: "region" }).chain(
                (dst.clone(), update.clone())
                    .shrink()
                    .map(|(dst, update)| Self::Lateral { dst, update }),
            ),
        ),
        Self::Call {
            ref detour,
            ref dst,
            ref combine,
            ..
        } => Box::new(dst.as_ref().shrink().chain(
            (detour.clone(), dst.clone(), combine.clone()).shrink().map(
                |(detour, dst, combine)| Self::Call {
                    region: "region",
                    detour,
                    dst,
                    combine,
                },
            ),
        )),
    }
});

impl<C: Ctrl<u8>> State<u8, C> {
    /// Construct an arbitrary value given an automaton with this many states.
    #[inline]
    #[must_use]
    pub fn arbitrary_given(n_states: NonZeroUsize, g: &mut Gen) -> Self {
        Self {
            transitions: Curry::arbitrary_given(n_states, g),
            non_accepting: BTreeSet::arbitrary(g),
        }
    }
}

impl<C: Ctrl<u8>> Curry<u8, C> {
    /// Construct an arbitrary value given an automaton with this many states.
    #[inline]
    #[must_use]
    pub fn arbitrary_given(n_states: NonZeroUsize, g: &mut Gen) -> Self {
        if bool::arbitrary(g) {
            Self::Wildcard(Transition::arbitrary_given(n_states, g))
        } else {
            Self::Scrutinize {
                filter: RangeMap::arbitrary_given(n_states, g),
                fallback: bool::arbitrary(g).then(|| Transition::arbitrary_given(n_states, g)),
            }
        }
    }
}

impl<C: Ctrl<u8>> RangeMap<u8, C> {
    /// Construct an arbitrary value given an automaton with this many states.
    #[inline]
    #[must_use]
    pub fn arbitrary_given(n_states: NonZeroUsize, g: &mut Gen) -> Self {
        let mut entries: BTreeMap<_, _> = (0..within_size(g))
            .map(|_| {
                (
                    Range::arbitrary(g),
                    Transition::arbitrary_given(n_states, g),
                )
            })
            .collect();
        // Remove overlap
        while let Some(key) = entries.keys().fold(None, |opt, k| {
            opt.or_else(|| {
                entries.range(..*k).fold(None, |acc, (range, _)| {
                    acc.or_else(|| range.intersection(*k).is_some().then_some(*k))
                })
            })
        }) {
            drop(entries.remove(&key));
        }
        Self(entries)
    }
}

impl<C: Ctrl<u8>> Transition<u8, C> {
    /// Construct an arbitrary value given an automaton with this many states.
    #[inline]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn arbitrary_given(n_states: NonZeroUsize, g: &mut Gen) -> Self {
        let choices: [fn(_, &mut _) -> _; 3] = [
            |n, r| Self::Lateral {
                dst: C::arbitrary_given(n, r),
                update: Arbitrary::arbitrary(r),
            },
            |n, r| Self::Call {
                region: "region",
                detour: C::arbitrary_given(n, r),
                dst: Box::new(Transition::arbitrary_given(n, r)),
                combine: Arbitrary::arbitrary(r),
            },
            |_, _| Self::Return { region: "region" },
        ];
        g.choose(&choices).expect("impossible")(n_states, g)
    }
}

impl Arbitrary for FF {
    #[inline]
    fn arbitrary(_: &mut Gen) -> Self {
        Self {
            lhs_t: "()".to_owned(),
            rhs_t: "()".to_owned(),
            output_t: "()".to_owned(),
            src: "|(), ()| ()".to_owned(),
        }
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(iter::empty())
    }
}
