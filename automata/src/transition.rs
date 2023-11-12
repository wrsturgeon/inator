/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Transition in an automaton: an action and a destination state.

use crate::{Ctrl, Input, InputError, ParseError, Update, FF};
use core::cmp;

// TODO: rename `Call` to `Open` and `Return` to `Close`

/// Transition in an automaton: an action and a destination state.
#[derive(Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum Transition<I: Input, C: Ctrl<I>> {
    /// Neither push nor pop: just move to a different state.
    Lateral {
        /// Go to this state.
        dst: C,
        /// Call this Rust function to update the output we're building.
        update: Option<Update<I>>,
    },
    /// Call another function--i.e., push a pointer/index onto the stack.
    Call {
        /// Region (user-defined name) that we're opening. Sensible to be e.g. "parentheses" for `(...)`.
        region: &'static str,
        /// Call (and require a successful run from) this state before continuing.
        detour: C,
        /// After the call has succeeded, go to this state.
        dst: C,
        /// Combine the cached results and the results of the called parser with this function.
        combine: FF,
    },
    /// Return into the function that called us.
    /// Note that this is NOT how we return from the overall parser:
    /// that happens only when input ends AND the stack is empty.
    Return {
        /// Region (user-defined name) that we're closing. Sensible to be e.g. "parentheses" for `(...)`.
        region: &'static str,
    },
}

impl<I: Input, C: Ctrl<I>> Clone for Transition<I, C> {
    #[inline]
    fn clone(&self) -> Self {
        match *self {
            Self::Lateral {
                ref dst,
                ref update,
            } => Self::Lateral {
                dst: dst.clone(),
                update: update.clone(),
            },
            Self::Call {
                region,
                ref detour,
                ref dst,
                ref combine,
            } => Self::Call {
                region,
                detour: detour.clone(),
                dst: dst.clone(),
                combine: combine.clone(),
            },
            Self::Return { region } => Self::Return { region },
        }
    }
}

impl<I: Input, C: Ctrl<I>> PartialEq for Transition<I, C> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq() // <-- TODO: faster
    }
}
impl<I: Input, C: Ctrl<I>> Eq for Transition<I, C> {}

impl<I: Input, C: Ctrl<I>> Ord for Transition<I, C> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match (self, other) {
            (&Self::Return { region: l_region }, &Self::Return { region: r_region }) => {
                l_region.cmp(r_region)
            }
            (&Self::Return { .. }, _) => cmp::Ordering::Less,
            (_, &Self::Return { .. }) => cmp::Ordering::Greater,
            (
                &Self::Lateral {
                    dst: ref l_dst,
                    update: ref l_update,
                },
                &Self::Lateral {
                    dst: ref r_dst,
                    update: ref r_update,
                },
            ) => l_dst.cmp(r_dst).then_with(|| l_update.cmp(r_update)),
            (&Self::Lateral { .. }, _) => cmp::Ordering::Less,
            (_, &Self::Lateral { .. }) => cmp::Ordering::Greater,
            (
                &Self::Call {
                    region: l_region,
                    detour: ref l_detour,
                    dst: ref l_dst,
                    combine: ref l_combine,
                },
                &Self::Call {
                    region: r_region,
                    detour: ref r_detour,
                    dst: ref r_dst,
                    combine: ref r_combine,
                },
            ) => l_region
                .cmp(r_region)
                .then_with(|| l_detour.cmp(r_detour))
                .then_with(|| l_dst.cmp(r_dst))
                .then_with(|| l_combine.cmp(r_combine)),
        }
    }
}

impl<I: Input, C: Ctrl<I>> PartialOrd for Transition<I, C> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<I: Input, C: Ctrl<I>> Transition<I, C> {
    /// Take this transition in an actual execution.
    /// Return the index of the machine's state after this transition.
    /// # Errors
    /// If we try to pop from an empty stack.
    #[inline]
    pub fn invoke(
        &self,
        output_t: &str,
        stack: &mut Vec<C>,
    ) -> Result<Option<(C, String)>, ParseError<I, C>> {
        match *self {
            Self::Lateral {
                ref dst,
                ref update,
            } => Ok(Some((
                dst.clone(),
                update.invoke(output_t).map_err(ParseError::BadParser)?,
            ))),
            Self::Call {
                ref detour,
                ref dst,
                ..
            } => {
                stack.push(dst.clone());
                Ok(Some((detour.clone(), "()".to_owned())))
            }
            Self::Return { .. } => {
                let rtn_to = stack
                    .pop()
                    .ok_or(ParseError::BadInput(InputError::Unopened))?;
                Ok(Some((rtn_to, output_t.to_owned())))
            }
        }
    }

    /// Compute the input type of any run that reaches this state.
    #[inline]
    #[must_use]
    pub fn input_type(&self) -> Option<&str> {
        match *self {
            Self::Lateral { ref update, .. } => Some(&update.input_t),
            Self::Call { ref combine, .. } => Some(&combine.lhs_t),
            Self::Return { .. } => None,
        }
    }

    /// Immediate next destination (as a state index).
    /// For local transitions, it's what you would expect.
    /// For calls, it's both the call and the continuation after the call.
    /// For returns, it's nothing.
    #[inline]
    #[must_use]
    pub fn dsts(&self) -> Vec<&C> {
        match *self {
            Self::Lateral { ref dst, .. } => vec![dst],
            Self::Call {
                ref detour,
                ref dst,
                ..
            } => vec![detour, dst],
            Self::Return { .. } => vec![],
        }
    }

    /// Natural-language representation of the action we're taking on the stack.
    #[inline]
    #[must_use]
    pub const fn in_english(&self) -> &'static str {
        match *self {
            Self::Lateral { .. } => "leave the stack alone",
            Self::Call { .. } => "push a call onto the stack",
            Self::Return { .. } => "return (i.e. pop from the stack)",
        }
    }
}

impl<I: Input> Transition<I, usize> {
    /// Convert the control parameter from `usize` to anything else.
    #[inline]
    #[must_use]
    pub fn convert_ctrl<C: Ctrl<I>>(self) -> Transition<I, C> {
        match self {
            Self::Lateral { dst, update } => Transition::Lateral {
                dst: C::from_usize(dst),
                update,
            },
            Self::Call {
                region,
                detour,
                dst,
                combine,
            } => Transition::Call {
                region,
                detour: C::from_usize(detour),
                dst: C::from_usize(dst),
                combine,
            },
            Self::Return { region } => Transition::Return { region },
        }
    }
}
