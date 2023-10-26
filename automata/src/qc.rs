/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! `QuickCheck` implementations for various types.

use crate::{
    Action, Ctrl, CurryInput, CurryStack, Graph, Input, Range, RangeMap, Stack, State, Transition,
    Update,
};
use core::{iter, num::NonZeroUsize};
use quickcheck::{Arbitrary, Gen};
use std::collections::BTreeMap;

impl<S: Arbitrary + Stack> Arbitrary for Action<S> {
    #[inline]
    fn arbitrary(g: &mut Gen) -> Self {
        match u8::arbitrary(g) % 3 {
            0 => Self::Local,
            1 => Self::Push(S::arbitrary(g)),
            2 => Self::Pop,
            _ => never!(),
        }
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match *self {
            Self::Local => Box::new(iter::empty()),
            Self::Pop => Box::new(iter::once(Self::Local)),
            Self::Push(ref symbol) => Box::new(
                [Self::Local, Self::Pop]
                    .into_iter()
                    .chain(symbol.shrink().map(Self::Push)),
            ),
        }
    }
}

/// Sample a value uniformly below the maximum size allowed by a generator.
#[inline]
#[allow(clippy::arithmetic_side_effects)]
fn within_size(g: &mut Gen) -> usize {
    usize::arbitrary(g) % NonZeroUsize::new(g.size()).expect("Zero-sized QuickCheck generator")
}

impl<S: Arbitrary + Stack, C: Arbitrary + Ctrl<u8, S, u8>> Arbitrary for Graph<u8, S, u8, C> {
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

impl<I: Input> Arbitrary for Update<I> {
    #[inline(always)]
    fn arbitrary(g: &mut Gen) -> Self {
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
        impl<
                S: Arbitrary + Stack,
                C: Arbitrary + Ctrl<u8, S, u8>,
            > Arbitrary for $t<u8, S, u8, C>
        {
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
    (self.transitions.clone(), self.accepting, self.tag.clone())
        .shrink()
        .map(|(transitions, accepting, tag)| Self {
            transitions,
            accepting,
            tag
        })
));

shrink_only!(|self: &CurryStack| Box::new(
    (
        self.wildcard.clone(),
        self.map_none.clone(),
        self.map_some.clone()
    )
        .shrink()
        .map(|(wildcard, map_none, map_some)| Self {
            wildcard,
            map_none,
            map_some
        })
));

shrink_only!(|self: &RangeMap| Box::new(self.entries.shrink().map(|entries| Self { entries })));

shrink_only!(|self: &CurryInput| match *self {
    Self::Wildcard(ref etc) => Box::new(etc.shrink().map(Self::Wildcard)),
    Self::Scrutinize(ref etc) => Box::new(
        etc.entries
            .first_key_value()
            .map(|(_, transition)| Self::Wildcard(transition.clone()))
            .into_iter()
            .chain(etc.shrink().map(Self::Scrutinize))
    ),
});

shrink_only!(|self: &Transition| Box::new(
    (self.dst.clone(), self.act.clone(), self.update)
        .shrink()
        .map(|(dst, act, update)| Self { dst, act, update })
));

impl<S: Arbitrary + Stack, C: Ctrl<u8, S, u8>> State<u8, S, u8, C> {
    /// Construct an arbitrary value given an automaton with this many states.
    #[inline]
    #[must_use]
    pub fn arbitrary_given(n_states: NonZeroUsize, g: &mut Gen) -> Self {
        Self {
            transitions: CurryStack::arbitrary_given(n_states, g),
            accepting: bool::arbitrary(g),
            tag: Vec::arbitrary(g),
        }
    }
}

impl<S: Arbitrary + Stack, C: Ctrl<u8, S, u8>> CurryStack<u8, S, u8, C> {
    /// Construct an arbitrary value given an automaton with this many states.
    #[inline]
    #[must_use]
    pub fn arbitrary_given(n_states: NonZeroUsize, g: &mut Gen) -> Self {
        let wildcard = bool::arbitrary(g).then(|| CurryInput::arbitrary_given(n_states, g));
        let mut map_none = bool::arbitrary(g).then(|| CurryInput::arbitrary_given(n_states, g));
        let mut map_some: BTreeMap<_, _> = (0..within_size(g))
            .map(|_| (S::arbitrary(g), CurryInput::arbitrary_given(n_states, g)))
            .collect();
        if let Some(ref wild) = wildcard {
            for some in map_some.values_mut() {
                while let Err((overlap, _, _)) = wild.disjoint(&*some) {
                    some.remove(overlap);
                }
            }
            if let Some(ref mut none) = map_none {
                while let Err((overlap, _, _)) = wild.disjoint(none) {
                    none.remove(overlap);
                }
            }
        }
        Self {
            wildcard,
            map_none,
            map_some,
        }
    }
}

impl<S: Arbitrary + Stack, C: Ctrl<u8, S, u8>> CurryInput<u8, S, u8, C> {
    /// Construct an arbitrary value given an automaton with this many states.
    #[inline]
    #[must_use]
    pub fn arbitrary_given(n_states: NonZeroUsize, g: &mut Gen) -> Self {
        if bool::arbitrary(g) {
            Self::Wildcard(Transition::arbitrary_given(n_states, g))
        } else {
            Self::Scrutinize(RangeMap::arbitrary_given(n_states, g))
        }
    }
}

impl<S: Arbitrary + Stack, C: Ctrl<u8, S, u8>> RangeMap<u8, S, u8, C> {
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
        Self { entries }
    }
}

impl<S: Arbitrary + Stack, C: Ctrl<u8, S, u8>> Transition<u8, S, u8, C> {
    /// Construct an arbitrary value given an automaton with this many states.
    #[inline]
    #[must_use]
    pub fn arbitrary_given(n_states: NonZeroUsize, g: &mut Gen) -> Self {
        Self {
            dst: C::arbitrary_given(n_states, g),
            act: Action::arbitrary(g),
            update: Update::arbitrary(g),
        }
    }
}
