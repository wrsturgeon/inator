/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! User-defined functions to call on transitions.

#[cfg(feature = "quickcheck")]
use quickcheck::*;

/// A user-defined function to call on a transition.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct Call {
    /// Name of the function, which becomes an identifier only AFTER `build.rs`.
    pub(crate) name: String, // `String` instead of `&'static str` for property-testing
    /// Whether the function inspects an input token at runtime.
    pub(crate) takes_arg: bool,
}

#[cfg(feature = "quickcheck")]
impl Arbitrary for Call {
    #[inline]
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            name: Arbitrary::arbitrary(g),
            takes_arg: bool::arbitrary(g),
        }
    }

    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.name.clone(), self.takes_arg)
                .shrink()
                .map(|(name, takes_arg)| Self { name, takes_arg }),
        )
    }
}
