/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! `QuickCheck` implementations for various types.

use crate::{
    Action, CmpFirst, Ctrl, CurryInput, CurryStack, Graph, Input, Range, RangeMap, Stack, State,
    Transition, Update,
};
use core::{iter, num::NonZeroUsize};
use quickcheck::{Arbitrary, Gen};

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
    NonZeroUsize::new(g.size()).map_or(0, |nz| usize::arbitrary(g) % nz)
}

impl<S: Arbitrary + Stack, C: Arbitrary + Ctrl<u8, S, u8>> Arbitrary for Graph<u8, S, u8, C> {
    #[inline]
    fn arbitrary(g: &mut Gen) -> Self {
        let size = within_size(g);
        let initial = C::arbitrary_given(size, g, false);
        if let Some(nz) = NonZeroUsize::new(size) {
            Self {
                states: (0..size)
                    .map(|_| State::arbitrary_given(nz, g, false))
                    .collect(),
                initial,
            }
        } else {
            Self {
                states: vec![],
                initial,
            }
        }
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.states.clone(), self.initial.clone())
                .shrink()
                .map(|(states, initial)| Self { states, initial }),
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

/// Finite set of functions that add or subtract powers of two, up to the limits of a type.
const UPDATE_CHOICES: [Update<u8, u8>; 17] = [
    update!(|x, _| x),
    update!(|x, _| x.saturating_add(1)),
    update!(|x, _| x.saturating_sub(1)),
    update!(|x, _| x.saturating_add(2)),
    update!(|x, _| x.saturating_sub(2)),
    update!(|x, _| x.saturating_add(4)),
    update!(|x, _| x.saturating_sub(4)),
    update!(|x, _| x.saturating_add(8)),
    update!(|x, _| x.saturating_sub(8)),
    update!(|x, _| x.saturating_add(16)),
    update!(|x, _| x.saturating_sub(16)),
    update!(|x, _| x.saturating_add(32)),
    update!(|x, _| x.saturating_sub(32)),
    update!(|x, _| x.saturating_add(64)),
    update!(|x, _| x.saturating_sub(64)),
    update!(|x, _| x.saturating_add(128)),
    update!(|x, _| x.saturating_sub(128)),
];

impl Arbitrary for Update<u8, u8> {
    #[inline(always)]
    fn arbitrary(g: &mut Gen) -> Self {
        *unwrap!(g.choose(&UPDATE_CHOICES))
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let i = unwrap!(UPDATE_CHOICES.iter().position(|u| u == self));
        Box::new(get!(UPDATE_CHOICES, ..i).iter().copied())
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

shrink_only!(
    |self: &State| Box::new((self.transitions.clone(), self.accepting).shrink().map(
        |(transitions, accepting)| Self {
            transitions,
            accepting
        }
    ))
);

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
            .first()
            .map(|&CmpFirst(_, ref transition)| Self::Wildcard(transition.clone()))
            .into_iter()
            .chain(etc.shrink().map(Self::Scrutinize))
    ),
});

shrink_only!(|self: &Transition| Box::new(
    (self.dst.clone(), self.act.clone(), self.update)
        .shrink()
        .map(|(dst, act, update)| Self { dst, act, update })
));

impl<K: Arbitrary + Ord, V: Arbitrary + Eq> Arbitrary for CmpFirst<K, V> {
    #[inline(always)]
    fn arbitrary(_: &mut Gen) -> Self {
        never!()
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.0.clone(), self.1.clone())
                .shrink()
                .map(|(k, v)| Self(k, v)),
        )
    }
}

impl<S: Arbitrary + Stack, C: Ctrl<u8, S, u8>> State<u8, S, u8, C> {
    /// Construct an arbitrary value given an automaton with this many states.
    #[inline]
    #[must_use]
    fn arbitrary_given(n_states: NonZeroUsize, g: &mut Gen, well_formed: bool) -> Self {
        Self {
            transitions: CurryStack::arbitrary_given(n_states, g, well_formed),
            accepting: bool::arbitrary(g),
        }
    }
}

impl<S: Arbitrary + Stack, C: Ctrl<u8, S, u8>> CurryStack<u8, S, u8, C> {
    /// Construct an arbitrary value given an automaton with this many states.
    #[inline]
    #[must_use]
    fn arbitrary_given(n_states: NonZeroUsize, g: &mut Gen, well_formed: bool) -> Self {
        Self {
            wildcard: bool::arbitrary(g)
                .then(|| CurryInput::arbitrary_given(n_states, g, well_formed)),
            map_none: bool::arbitrary(g)
                .then(|| CurryInput::arbitrary_given(n_states, g, well_formed)),
            map_some: (0..within_size(g))
                .map(|_| {
                    (
                        S::arbitrary(g),
                        CurryInput::arbitrary_given(n_states, g, well_formed),
                    )
                })
                .collect(),
        }
    }
}

impl<S: Arbitrary + Stack, C: Ctrl<u8, S, u8>> CurryInput<u8, S, u8, C> {
    /// Construct an arbitrary value given an automaton with this many states.
    #[inline]
    #[must_use]
    fn arbitrary_given(n_states: NonZeroUsize, g: &mut Gen, well_formed: bool) -> Self {
        if bool::arbitrary(g) {
            Self::Wildcard(Transition::arbitrary_given(n_states, g, well_formed))
        } else {
            Self::Scrutinize(RangeMap::arbitrary_given(n_states, g, well_formed))
        }
    }
}

impl<S: Arbitrary + Stack, C: Ctrl<u8, S, u8>> RangeMap<u8, S, u8, C> {
    /// Construct an arbitrary value given an automaton with this many states.
    #[inline]
    #[must_use]
    fn arbitrary_given(n_states: NonZeroUsize, g: &mut Gen, well_formed: bool) -> Self {
        Self {
            entries: (0..within_size(g))
                .map(|_| {
                    CmpFirst(
                        Range::arbitrary(g),
                        Transition::arbitrary_given(n_states, g, well_formed),
                    )
                })
                .collect(),
        }
    }
}

impl<S: Arbitrary + Stack, C: Ctrl<u8, S, u8>> Transition<u8, S, u8, C> {
    /// Construct an arbitrary value given an automaton with this many states.
    #[inline]
    #[must_use]
    fn arbitrary_given(n_states: NonZeroUsize, g: &mut Gen, well_formed: bool) -> Self {
        Self {
            dst: C::arbitrary_given(n_states.into(), g, well_formed),
            act: Action::arbitrary(g),
            update: Update::arbitrary(g),
        }
    }
}
