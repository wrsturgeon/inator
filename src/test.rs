/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(
    clippy::absolute_paths,
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]

#[cfg(feature = "quickcheck")] // <-- TODO: disable for reduced tests
use crate::*;

/*
/// Check if we can split this input into a bunch of non-zero-sized slices
/// that are all individually accepted by a given parser.
#[inline]
#[cfg(feature = "quickcheck")] // <-- TODO: disable for reduced tests
fn sliceable<I: Input, C: Ctrl<I>>(parser: &Graph<I, C>, input: &[I]) -> bool {
    input.is_empty()
        || (1..=input.len()).rev().any(|i| {
            parser.accept(input[..i].iter().cloned()).is_ok() && sliceable(parser, &input[i..])
        })
}
*/

#[cfg(feature = "quickcheck")]
mod prop {
    use super::*;
    use quickcheck::*;

    quickcheck! {
        fn empty_works(input: Vec<u8>) -> bool {
            let parser = empty::<u8>();
            if parser.check().is_err() { return false; }
            input.is_empty() == empty().accept(input).is_ok()
        }

        fn any_of_works(range: Range<u8>, input: Vec<u8>) -> bool {
            let parser = any_of(range, update!(|(), _| {}));
            if parser.check().is_err() { return false; }
            parser.accept(input.iter().copied()).is_ok() == (input.len() == 1 && range.contains(&input[0]))
        }

        fn star_in_paren(count: u8) -> bool {
            let input = iter::once('(').chain(iter::repeat('a').take(usize::from(count))).chain(iter::once(')'));
            let parser = open("parentheses", c('(')) >> c('a').star() << close("parentheses", c(')'));
            parser.accept(input).is_ok()
        }
    }
}
