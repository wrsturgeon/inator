/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Closed ranges that implement `Ord`.

use crate::Expression;
use core::{fmt::Debug, ops::RangeInclusive};
use proc_macro2::Span;
use rand::{distributions::uniform::SampleUniform, rngs::ThreadRng, Rng};
use syn::{Expr, ExprRange, Pat, RangeLimits, Token, Type};

#[cfg(feature = "quickcheck")]
use quickcheck::*;

/// Closed range that implements `Ord`.
#[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Range<T: Clone + Ord> {
    /// Start of the range: `a` in `a..=b`.
    pub start: T,
    /// End of the range: `b` in `a..=b`.
    pub end: T,
}

impl<T: Clone + Ord> Range<T> {
    /// Range with one value (e.g. given `1`, returns `1..=1`).
    #[must_use]
    #[inline(always)]
    pub fn singleton(only: T) -> Self {
        Self {
            start: only.clone(),
            end: only,
        }
    }

    /// Check if a value is in this range.
    #[must_use]
    #[inline(always)]
    pub fn contains(&self, value: &T) -> bool {
        (self.start.clone()..=self.end.clone()).contains(value)
    }

    /// Get a random element in this range.
    #[must_use]
    #[inline(always)]
    pub fn random(&self, rng: &mut ThreadRng) -> T
    where
        T: SampleUniform,
    {
        rng.gen_range(self.start.clone()..=self.end.clone())
    }
}

impl<T: Clone + Expression + Ord> Expression for Range<T> {
    #[inline]
    fn to_expr(&self) -> Expr {
        Expr::Range(ExprRange {
            attrs: vec![],
            start: Some(Box::new(self.start.to_expr())),
            limits: RangeLimits::Closed(Token![..=](Span::call_site())),
            end: Some(Box::new(self.end.to_expr())),
        })
    }
    #[inline]
    fn to_pattern(&self) -> Pat {
        Pat::Range(ExprRange {
            attrs: vec![],
            start: Some(Box::new(self.start.to_expr())),
            limits: RangeLimits::Closed(Token![..=](Span::call_site())),
            end: Some(Box::new(self.end.to_expr())),
        })
    }
    #[inline(always)]
    fn to_type() -> Type {
        T::to_type()
    }
}

impl<T: Clone + Ord> From<RangeInclusive<T>> for Range<T> {
    #[inline]
    fn from(value: RangeInclusive<T>) -> Self {
        Self {
            start: value.start().clone(),
            end: value.end().clone(),
        }
    }
}

impl<T: Clone + Debug + Ord> Debug for Range<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}..={:?}", self.start, self.end)
    }
}

#[cfg(feature = "quickcheck")]
impl<T: Arbitrary + Clone + Ord> Arbitrary for Range<T> {
    #[inline]
    fn arbitrary(g: &mut Gen) -> Self {
        let (a, b) = (T::arbitrary(g), T::arbitrary(g));
        Self {
            start: a.clone().min(b.clone()),
            end: a.max(b),
        }
    }

    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.start.clone(), self.end.clone())
                .shrink()
                .map(|(a, b)| Self {
                    start: a.clone().min(b.clone()),
                    end: a.max(b),
                }),
        )
    }
}
