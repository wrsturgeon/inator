/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! `QuickCheck` implementations for various types.

use crate::{Action, Input, Range, Stack};
use core::iter;
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
