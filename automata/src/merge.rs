/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Trait to fallibly combine multiple values into one value with identical semantics.

use crate::{
    Ctrl, CtrlMergeConflict, Curry, IllFormed, Input, RangeMap, State, Transition, Update, FF,
};
use core::convert::Infallible;
use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};

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

impl<'s> Merge for &'s str {
    type Error = (&'s str, &'s str);
    #[inline]
    fn merge(self, other: Self) -> Result<Self, Self::Error> {
        if self == other {
            Ok(self)
        } else {
            Err((self, other))
        }
    }
}

impl<T: Ord> Merge for BTreeSet<T> {
    type Error = CtrlMergeConflict;
    #[inline]
    fn merge(mut self, other: Self) -> Result<Self, Self::Error> {
        self.extend(other);
        Ok(self)
    }
}

impl<K: Clone + Ord, V: Merge> Merge for BTreeMap<K, V> {
    type Error = (K, V::Error);
    #[inline]
    fn merge(mut self, other: Self) -> Result<Self, Self::Error> {
        for (k, v) in other {
            match self.entry(k) {
                Entry::Occupied(extant) => {
                    let (lk, lv) = extant.remove_entry();
                    let mv = lv.merge(v).map_err(|e| (lk.clone(), e))?;
                    drop(self.insert(lk, mv));
                }
                Entry::Vacant(empty) => drop(empty.insert(v)),
            }
        }
        Ok(self)
    }
}

impl<I: Input, C: Ctrl<I>> Merge for State<I, C> {
    type Error = IllFormed<I, C>;
    #[inline]
    #[allow(clippy::unwrap_in_result)]
    fn merge(mut self, other: Self) -> Result<Self, Self::Error> {
        Ok(Self {
            transitions: self.transitions.merge(other.transitions)?,
            non_accepting: if self.non_accepting.is_empty() || other.non_accepting.is_empty() {
                BTreeSet::new()
            } else {
                self.non_accepting.extend(other.non_accepting);
                self.non_accepting
            },
        })
    }
}

impl<T> Merge for Vec<T> {
    type Error = Infallible;
    #[inline]
    fn merge(mut self, other: Self) -> Result<Self, Self::Error> {
        self.extend(other);
        Ok(self)
    }
}

impl<I: Input, C: Ctrl<I>> Merge for Curry<I, C> {
    type Error = IllFormed<I, C>;
    #[inline]
    fn merge(self, other: Self) -> Result<Self, Self::Error> {
        match (self, other) {
            (Self::Wildcard(lhs), Self::Wildcard(rhs)) => Ok(Self::Wildcard(lhs.merge(rhs)?)),
            (Self::Wildcard(w), Self::Scrutinize(s)) | (Self::Scrutinize(s), Self::Wildcard(w)) => {
                match s.0.first_key_value() {
                    None => Ok(Self::Wildcard(w)),
                    Some((k, v)) => Err(IllFormed::WildcardMask {
                        arg_token: Some(k.clone()),
                        possibility_1: Box::new(w),
                        possibility_2: Box::new(v.clone()),
                    }),
                }
            }
            (Self::Scrutinize(lhs), Self::Scrutinize(rhs)) => Ok(Self::Scrutinize(lhs.merge(rhs)?)),
        }
    }
}

impl<I: Input, C: Ctrl<I>> Merge for RangeMap<I, C> {
    type Error = IllFormed<I, C>;
    #[inline]
    fn merge(self, other: Self) -> Result<Self, Self::Error> {
        Ok(Self(self.0.merge(other.0).map_err(|(_, e)| e)?))
    }
}

impl<I: Input, C: Ctrl<I>> Merge for Transition<I, C> {
    type Error = IllFormed<I, C>;
    #[inline]
    fn merge(self, other: Self) -> Result<Self, Self::Error> {
        match (self, other) {
            (
                Self::Lateral {
                    dst: l_dst,
                    update: l_update,
                },
                Self::Lateral {
                    dst: r_dst,
                    update: r_update,
                },
            ) => Ok(Self::Lateral {
                dst: l_dst.merge(r_dst).map_err(|e| match e {
                    CtrlMergeConflict::NotEqual(a, b) => IllFormed::Superposition(a, b),
                })?,
                update: l_update
                    .merge(r_update)
                    .map_err(|(a, b)| IllFormed::IncompatibleCallbacks(Box::new(a), Box::new(b)))?,
            }),
            (
                Self::Call {
                    region: l_region,
                    detour: l_detour,
                    dst: l_dst,
                    combine: l_combine,
                },
                Self::Call {
                    region: r_region,
                    detour: r_detour,
                    dst: r_dst,
                    combine: r_combine,
                },
            ) => Ok(Self::Call {
                region: l_region
                    .merge(r_region)
                    .map_err(|(a, b)| IllFormed::AmbiguousRegions(a, b))?,
                detour: l_detour.merge(r_detour).map_err(|e| match e {
                    CtrlMergeConflict::NotEqual(a, b) => IllFormed::Superposition(a, b),
                })?,
                dst: l_dst.merge(r_dst).map_err(|e| match e {
                    CtrlMergeConflict::NotEqual(a, b) => IllFormed::Superposition(a, b),
                })?,
                combine: l_combine.merge(r_combine).map_err(|(a, b)| {
                    IllFormed::IncompatibleCombinators(Box::new(a), Box::new(b))
                })?,
            }),
            (Self::Return { region: l_region }, Self::Return { region: r_region }) => {
                Ok(Self::Return {
                    region: l_region
                        .merge(r_region)
                        .map_err(|(a, b)| IllFormed::AmbiguousRegions(a, b))?,
                })
            }
            (a, b) => Err(IllFormed::IncompatibleActions(Box::new(a), Box::new(b))),
        }
    }
}

impl<I: Input> Merge for Update<I> {
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

impl Merge for FF {
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
