/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Trait to fallibly combine multiple values into one value with identical semantics.

use crate::{
    Action, CmpFirst, Ctrl, CtrlMergeConflict, CurryInput, CurryStack, IllFormed, Input, Output,
    Range, RangeMap, Stack, State, Transition, Update,
};
use std::collections::{btree_map, BTreeMap, BTreeSet};

/// Trait to fallibly combine multiple values into one value with identical semantics.
pub trait Merge: Sized {
    /// Implementation-defined error providing a witness to the reason the merge failed.
    type Error;
    /// Fallibly combine multiple values into one value with identical semantics.
    /// # Errors
    /// Implementation-defined: if the merge as we define it can't work.
    fn merge(self, other: Self) -> Result<Self, Self::Error>;
}

/// Merge a collection of elements into one.
/// Return `None` if the collection was empty.
#[inline]
pub fn merge<M: Merge, In: IntoIterator<Item = M>>(input: In) -> Option<Result<M, M::Error>> {
    let mut iter = input.into_iter();
    let first = iter.next()?;
    Some(iter.try_fold(first, Merge::merge))
}

/// Merge a collection of `Result`s possibly containing elements into one.
/// Return `None` if the collection was empty.
#[inline]
#[allow(clippy::module_name_repetitions)]
pub fn try_merge<M: Merge, In: IntoIterator<Item = Result<M, M::Error>>>(
    input: In,
) -> Option<Result<M, M::Error>> {
    let mut iter = input.into_iter();
    iter.next()?.map_or_else(
        |e| Some(Err(e)),
        |first| Some(iter.try_fold(first, |acc, m| acc.merge(m?))),
    )
}

impl<T> Merge for Option<T> {
    type Error = (T, T);
    #[inline]
    fn merge(self, other: Self) -> Result<Self, Self::Error> {
        match (self, other) {
            (None, None) => Ok(None),
            (Some(a), None) => Ok(Some(a)),
            (None, Some(b)) => Ok(Some(b)),
            (Some(a), Some(b)) => Err((a, b)),
        }
    }
}

impl Merge for usize {
    type Error = CtrlMergeConflict;
    #[inline]
    fn merge(self, other: Self) -> Result<Self, Self::Error> {
        if self == other {
            Ok(self)
        } else {
            Err(CtrlMergeConflict::NotEqual(self, other))
        }
    }
}

impl Merge for BTreeSet<usize> {
    type Error = CtrlMergeConflict;
    #[inline]
    fn merge(mut self, other: Self) -> Result<Self, Self::Error> {
        self.extend(other);
        Ok(self)
    }
}

impl<K: Ord, V: Merge> Merge for BTreeMap<K, V> {
    type Error = V::Error;
    #[inline]
    fn merge(mut self, other: Self) -> Result<Self, Self::Error> {
        for (k, v) in other {
            match self.entry(k) {
                btree_map::Entry::Occupied(extant) => {
                    let (lk, lv) = extant.remove_entry();
                    drop(self.insert(lk, lv.merge(v)?));
                }
                btree_map::Entry::Vacant(empty) => drop(empty.insert(v)),
            }
        }
        Ok(self)
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Merge for State<I, S, O, C> {
    type Error = IllFormed<I, S, O, C>;
    #[inline]
    fn merge(self, other: Self) -> Result<Self, Self::Error> {
        Ok(Self {
            transitions: self.transitions.merge(other.transitions)?,
            accepting: self.accepting || other.accepting,
        })
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Merge for CurryStack<I, S, O, C> {
    type Error = IllFormed<I, S, O, C>;
    #[inline]
    fn merge(self, other: Self) -> Result<Self, Self::Error> {
        Ok(Self {
            wildcard: self
                .wildcard
                .merge(other.wildcard)
                .or_else(|(a, b)| Ok(Some(a.merge(b)?)))?,
            map_none: self
                .map_none
                .merge(other.map_none)
                .or_else(|(a, b)| Ok(Some(a.merge(b)?)))?,
            map_some: self.map_some.merge(other.map_some)?,
        })
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Merge for CurryInput<I, S, O, C> {
    type Error = IllFormed<I, S, O, C>;
    #[inline]
    #[allow(clippy::todo)] // <-- TODO
    fn merge(self, other: Self) -> Result<Self, Self::Error> {
        match (self, other) {
            (Self::Wildcard(lhs), Self::Wildcard(rhs)) => Ok(Self::Wildcard(lhs.merge(rhs)?)),
            (Self::Wildcard(w), Self::Scrutinize(s)) | (Self::Scrutinize(s), Self::Wildcard(w)) => {
                if s.entries.is_empty() {
                    Ok(Self::Wildcard(w))
                } else {
                    todo!()
                }
            }
            (Self::Scrutinize(lhs), Self::Scrutinize(rhs)) => Ok(Self::Scrutinize(lhs.merge(rhs)?)),
        }
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Merge for RangeMap<I, S, O, C> {
    type Error = IllFormed<I, S, O, C>;
    #[inline]
    fn merge(self, other: Self) -> Result<Self, Self::Error> {
        Ok(Self {
            entries: self.entries.merge(other.entries)?,
        })
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Merge
    for BTreeSet<CmpFirst<Range<I>, Transition<I, S, O, C>>>
{
    type Error = IllFormed<I, S, O, C>;
    #[inline]
    fn merge(self, other: Self) -> Result<Self, Self::Error> {
        let a: BTreeMap<_, _> = self.into_iter().map(|CmpFirst(k, v)| (k, v)).collect();
        let b: BTreeMap<_, _> = other.into_iter().map(|CmpFirst(k, v)| (k, v)).collect();
        Ok(a.merge(b)?
            .into_iter()
            .map(|(k, v)| CmpFirst(k, v))
            .collect())
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Merge for Transition<I, S, O, C> {
    type Error = IllFormed<I, S, O, C>;
    #[inline]
    fn merge(self, other: Self) -> Result<Self, Self::Error> {
        Ok(Self {
            dst: self
                .dst
                .merge(other.dst)
                .map_err(|e: CtrlMergeConflict| match e {
                    CtrlMergeConflict::NotEqual(a, b) => IllFormed::Superposition(a, b),
                })?,
            act: self
                .act
                .merge(other.act)
                .map_err(|(a, b): (Action<S>, Action<S>)| {
                    IllFormed::IncompatibleStackActions(a, b)
                })?,
            update: self.update.merge(other.update).map_err(
                |(a, b): (Update<I, O>, Update<I, O>)| IllFormed::IncompatibleCallbacks(a, b),
            )?,
        })
    }
}

impl<I: Input, O: Output> Merge for Update<I, O> {
    type Error = (Self, Self);
    #[inline]
    fn merge(self, other: Self) -> Result<Self, Self::Error> {
        if self == other {
            Ok(self)
        } else {
            Err((self, other))
        }
    }
}

impl<S: Stack> Merge for Action<S> {
    type Error = (Self, Self);
    #[inline]
    fn merge(self, other: Self) -> Result<Self, Self::Error> {
        if self == other {
            Ok(self)
        } else {
            Err((self, other))
        }
    }
}
