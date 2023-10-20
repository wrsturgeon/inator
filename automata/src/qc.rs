/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! `QuickCheck` implementations for various types.

use crate::{
    Action, Ctrl, CurryInput, CurryStack, Graph, Input, Output, Range, RangeMap, Stack, State,
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
        let initial = C::arbitrary_given(size, g);
        if let Some(nz) = NonZeroUsize::new(size) {
            Self {
                states: (0..size).map(|_| State::arbitrary_given(nz, g)).collect(),
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
                I: Arbitrary + Input,
                S: Arbitrary + Stack,
                O: 'static + Clone + Output,
                C: Arbitrary + Ctrl<I, S, O>,
            > Arbitrary for $t<I, S, O, C>
        {
            #[inline(always)]
            fn arbitrary(_: &mut Gen) -> Self {
                unreachable!()
            }
            #[inline]
            fn shrink(&$self) -> Box<dyn Iterator<Item = Self>> {
                $body
            }
        }
    };
}

shrink_only!(|self: &CurryInput| todo!());
shrink_only!(|self: &CurryStack| todo!());
shrink_only!(|self: &RangeMap| todo!());
shrink_only!(|self: &Transition| todo!());
shrink_only!(|self: &State| todo!());

impl<S: Arbitrary + Stack, C: Ctrl<u8, S, u8>> State<u8, S, u8, C> {
    /// Construct an arbitrary value given an automaton with this many states.
    #[inline]
    #[must_use]
    fn arbitrary_given(n_states: NonZeroUsize, g: &mut Gen) -> Self {
        Self {
            transitions: CurryStack::arbitrary_given(n_states, g),
            accepting: bool::arbitrary(g),
        }
    }
}

impl<S: Arbitrary + Stack, C: Ctrl<u8, S, u8>> CurryStack<u8, S, u8, C> {
    /// Construct an arbitrary value given an automaton with this many states.
    #[inline]
    #[must_use]
    fn arbitrary_given(n_states: NonZeroUsize, g: &mut Gen) -> Self {
        Self {
            wildcard: bool::arbitrary(g).then(|| CurryInput::arbitrary_given(n_states, g)),
            map_none: bool::arbitrary(g).then(|| CurryInput::arbitrary_given(n_states, g)),
            map_some: (0..within_size(g))
                .map(|_| (S::arbitrary(g), CurryInput::arbitrary_given(n_states, g)))
                .collect(),
        }
    }
}

impl<S: Arbitrary + Stack, C: Ctrl<u8, S, u8>> CurryInput<u8, S, u8, C> {
    /// Construct an arbitrary value given an automaton with this many states.
    #[inline]
    #[must_use]
    fn arbitrary_given(n_states: NonZeroUsize, g: &mut Gen) -> Self {
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
    fn arbitrary_given(n_states: NonZeroUsize, g: &mut Gen) -> Self {
        Self {
            entries: (0..within_size(g))
                .map(|_| {
                    (
                        Range::arbitrary(g),
                        Transition::arbitrary_given(n_states, g),
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
    fn arbitrary_given(n_states: NonZeroUsize, g: &mut Gen) -> Self {
        Self {
            dst: C::arbitrary_given(n_states.into(), g),
            act: Action::arbitrary(g),
            update: Update::arbitrary(g),
        }
    }
}
