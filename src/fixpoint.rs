/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Fixpoint: just a tagged state that can be called later.

use core::ops;
use inator_automata::*;

/// Tagged state that can be called later.
#[must_use = "Fixpoints do nothing unless applied to an automaton with the `>>` operator."]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Fixpoint<I: Input> {
    /// Tag that will be associated with the initial state of the right-hand argument to `>>`.
    tag: String,
    /// Anything to the left-hand side, e.g. from `... a >> b >> fixpoint("f") >> d`.
    etc: Option<Deterministic<I>>,
}

impl<I: Input> ops::Shr<Deterministic<I>> for Fixpoint<I> {
    type Output = Deterministic<I>;
    #[inline]
    #[allow(clippy::arithmetic_side_effects, clippy::manual_assert, clippy::panic)]
    fn shr(self, mut rhs: Deterministic<I>) -> Self::Output {
        if let Some(lhs) = self.etc {
            rhs = lhs >> rhs;
        }
        if rhs.tags.insert(self.tag, rhs.initial).is_some() {
            panic!("Fixpoint name already in use");
        }
        rhs
    }
}

impl<I: Input> ops::Shr<Fixpoint<I>> for Deterministic<I> {
    type Output = Fixpoint<I>;
    #[inline]
    #[allow(clippy::manual_assert, clippy::panic)]
    fn shr(self, mut rhs: Fixpoint<I>) -> Self::Output {
        assert!(
            rhs.etc.is_none(),
            "Called something of the form `a >> fixpoint(\"f\")`, then \
            tried to put something else to the left of that same fixpoint object."
        );
        rhs.etc = Some(self);
        rhs
    }
}

/// Name a point in code so we can call it later by name.
#[inline]
pub fn fixpoint<I: Input>(call_by_name: &str) -> Fixpoint<I> {
    Fixpoint {
        tag: call_by_name.to_owned(),
        etc: None,
    }
}
