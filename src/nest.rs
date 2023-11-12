/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Nest three parsers (one opens, one parses, one closes) into a named region.

use crate::*;
use core::{marker::PhantomData, mem};

pub trait Nest {
    /// Nest three parsers (one opens, one parses, one closes) into a named region.
    #[must_use]
    fn nest<I: Input>(
        &self,
        open: Parser<I>,
        inside: Parser<I>,
        close: Parser<I>,
        combine: FF,
    ) -> Parser<I>;
}

impl Nest for &'static str {
    #[inline]
    fn nest<I: Input>(
        &self,
        open: Parser<I>,
        inside: Parser<I>,
        close: Parser<I>,
        combine: FF,
    ) -> Parser<I> {
        // If `inside` could end in multiple places, use feedback from the closing delimiter to tell where.
        // Why not return *then* parser the delimiter? Because then e.g. `(a*)` would not be parseable:
        // e.g. `(a)` and `(aaaa)` would be indistinguishable until the end.
        let fused = {
            let mut size = open.states.len();
            (inside >> close)
                .map_indices(|i| i.checked_add(size).expect("Absurdly huge number of states"))
        };

        // Edit each accepting state in `open` to call `fused` instead.
        let call = Call {
            region: *self,
            init: fused.initial,
            combine,
            ghost: PhantomData,
        };
        let accepting = open
            .states
            .iter_mut()
            .filter(|s| mem::take(&mut s.non_accepting).is_empty());
        for s in accepting {
            match s.transitions {
                Curry::Wildcard(ref mut t) => t.calls.push(call),
                Curry::Scrutinize(ref mut map) => {
                    for t in map.0.values_mut() {
                        t.calls.push(call);
                    }
                }
            }
        }

        open.states.extend(fused.states);

        open
    }
}
