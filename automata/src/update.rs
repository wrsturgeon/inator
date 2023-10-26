/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! A single-argument Rust function callable both in `build.rs` and in a source file.

use crate::{Input, ToSrc};
use core::{cmp, fmt};
use std::any::Any;

/// A single-argument Rust function callable both in `build.rs` and in a source file.
#[allow(clippy::exhaustive_structs)]
pub struct Update<I: Input> {
    /// Source-code representation of the input type.
    pub input_t: String,
    /// Source-code representation of the input type.
    pub output_t: String,
    /// Source-code representation that's promised to compile to a call operationally identical to `ptr`.
    pub src: &'static str,
    /// Dynamically typed function to compute the update function.
    pub run: Box<dyn Fn(Box<dyn Any>, I) -> Box<dyn Any>>,
}

impl<I: Input> Update<I> {
    #[inline]
    #[must_use]
    pub fn _update_macro<T: ToSrc, U: ToSrc>(src: &'static str, ptr: fn(T, I) -> U) -> Self {
        let run = Box::new(move |dyn_acc: Box<dyn Any>, tok: I| -> Box<dyn Any> {
            let acc: T = *dyn_acc.downcast().expect("Type mismatch");
            let out: U = ptr(acc, tok);
            Box::new(out)
        });
        Self {
            input_t: T::src_type(),
            output_t: U::src_type(),
            src,
            run,
        }
    }
}

impl<I: Input> PartialEq for Update<I> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.src == other.src
    }
}

impl<I: Input> Eq for Update<I> {}

impl<I: Input> PartialOrd for Update<I> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<I: Input> Ord for Update<I> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.src.cmp(other.src)
    }
}

impl<I: Input> fmt::Debug for Update<I> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "update!({})", self.src)
    }
}

impl<I: Input> Clone for Update<I> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            input_t: self.input_t.clone(),
            output_t: self.output_t.clone(),
            src: self.src,
            run: self.run,
        }
    }
}
