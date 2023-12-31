/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

///////////////////////////////////////////////////////////////////////////////////////////////////
/////_________________________________________________________________________________/////////////
//////_____/\\\___/\\\\\___/\\\___/\\\\\\\\\\__/\\\\\\\\\__/\\\\\\\\\\___/\\\\\\\\\\___////////////
///////____\/\\\__\/\\\\\\__/\\\__\/\\\////\\\_\///\\\///__\/\\\////\\\__\/\\\////\\\___///////////
////////____\/\\\__\/\\\/\\\_/\\\__\/\\\__\/\\\___\/\\\_____\/\\\__\/\\\__\/\\\__\/\\\___//////////
/////////____\/\\\__\/\\\//\\\/\\\__\/\\\\\\\\\\___\/\\\_____\/\\\__\/\\\__\/\\\\\\\\/____/////////
//////////____\/\\\__\/\\\\//\\\\\\__\/\\\////\\\___\/\\\_____\/\\\__\/\\\__\/\\\///\\\____////////
///////////____\/\\\__\/\\\_\//\\\\\__\/\\\__\/\\\___\/\\\_____\/\\\\\\\\\\__\/\\\_\//\\\___///////
////////////____\///___\///___\/////___\///___\///____\///______\//////////___\/\\\__\///____//////
/////////////_________________________________________________________________________________/////
///////////////////////////////////////////////////////////////////////////////////////////////////

#![doc = include_str!("../README.md")]
#![deny(warnings)]
#![allow(unknown_lints)]
#![warn(
    clippy::all,
    clippy::missing_docs_in_private_items,
    clippy::nursery,
    clippy::pedantic,
    clippy::perf,
    clippy::restriction,
    clippy::cargo,
    elided_lifetimes_in_paths,
    missing_docs,
    rustdoc::all
)]
// https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html
#![warn(
    absolute_paths_not_starting_with_crate,
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    keyword_idents,
    let_underscore_drop,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    non_ascii_idents,
    noop_method_call,
    pointer_structural_match,
    rust_2021_incompatible_closure_captures,
    rust_2021_incompatible_or_patterns,
    rust_2021_prefixes_incompatible_syntax,
    rust_2021_prelude_collisions,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_code,
    unsafe_op_in_unsafe_fn,
    unstable_features,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_macro_rules,
    unused_qualifications,
    unused_results,
    unused_tuple_struct_fields,
    variant_size_differences
)]
#![allow(
    clippy::blanket_clippy_restriction_lints,
    clippy::cargo_common_metadata,
    clippy::expect_used,
    clippy::implicit_return,
    clippy::inline_always,
    clippy::let_underscore_untyped,
    clippy::min_ident_chars,
    clippy::missing_trait_methods,
    clippy::mod_module_files,
    clippy::multiple_unsafe_ops_per_block,
    clippy::needless_borrowed_reference,
    clippy::option_option,
    clippy::partial_pub_fields,
    clippy::pub_use,
    clippy::pub_with_shorthand,
    clippy::question_mark_used,
    clippy::redundant_pub_crate,
    clippy::ref_patterns,
    clippy::semicolon_outside_block,
    clippy::separated_literal_suffix,
    clippy::similar_names,
    clippy::single_call_fn,
    clippy::single_char_lifetime_names,
    clippy::std_instead_of_alloc,
    clippy::string_add,
    clippy::use_self,
    clippy::wildcard_imports
)]

/*
/// Unwrap if we're debugging but `unwrap_unchecked` if we're not.
#[cfg(any(debug_assertions, test))]
macro_rules! unwrap {
    ($expr:expr) => {
        $expr.unwrap()
    };
}

/// Unwrap if we're debugging but `unwrap_unchecked` if we're not.
#[cfg(not(any(debug_assertions, test)))]
macro_rules! unwrap {
    ($expr:expr) => {{
        #[allow(unsafe_code, unused_unsafe)]
        let result = unsafe { $expr.unwrap_unchecked() };
        result
    }};
}

/// Unwrap if we're debugging but `unwrap_unchecked` if we're not.
#[cfg(any(debug_assertions, test))]
macro_rules! get {
    ($expr:expr, $index:expr) => {
        $expr.get($index).unwrap()
    };
}

/// Unwrap if we're debugging but `unwrap_unchecked` if we're not.
#[cfg(not(any(debug_assertions, test)))]
macro_rules! get {
    ($expr:expr, $index:expr) => {{
        #[allow(unsafe_code, unused_unsafe)]
        let result = unsafe { $expr.get_unchecked($index) };
        result
    }};
}

/// Unwrap if we're debugging but `unwrap_unchecked` if we're not.
#[cfg(any(debug_assertions, test))]
macro_rules! get_mut {
    ($expr:expr, $index:expr) => {
        $expr.get_mut($index).unwrap()
    };
}

/// Unwrap if we're debugging but `unwrap_unchecked` if we're not.
#[cfg(not(any(debug_assertions, test)))]
macro_rules! get_mut {
    ($expr:expr, $index:expr) => {{
        #[allow(unsafe_code, unused_unsafe)]
        let result = unsafe { $expr.get_unchecked_mut($index) };
        result
    }};
}
*/

// TODO: derive ToSrc

#[cfg(test)]
mod test;

pub use inator_automata::{Deterministic as Parser, *};

use core::iter;
use std::collections::{BTreeMap, BTreeSet};

#[cfg(feature = "quickcheck")]
use quickcheck as _; // <-- TODO: remove if we write some implementations

/// Parser that accepts only the empty string.
#[inline]
#[must_use]
pub fn empty<I: Input>() -> Deterministic<I> {
    Graph {
        states: vec![State {
            transitions: Curry::Scrutinize {
                filter: RangeMap(BTreeMap::new()),
                fallback: None,
            },
            non_accepting: BTreeSet::new(),
        }],
        initial: 0,
    }
}

/// Accept exactly this range of tokens and do exactly these things.
#[inline]
#[must_use]
pub fn on_any_of<I: Input>(range: Range<I>, update: Update<I>) -> Deterministic<I> {
    Graph {
        states: vec![
            State {
                transitions: Curry::Scrutinize {
                    filter: RangeMap(BTreeMap::new()),
                    fallback: None,
                },
                non_accepting: BTreeSet::new(),
            },
            State {
                non_accepting: iter::once(format!(
                    "Expected only a single token on [{}..={}] but got another token after it",
                    range.first.to_src(),
                    range.last.to_src(),
                ))
                .collect(),
                transitions: Curry::Scrutinize {
                    filter: RangeMap(
                        iter::once((
                            range,
                            Transition::Lateral {
                                dst: 0,
                                update: Some(update),
                            },
                        ))
                        .collect(),
                    ),
                    fallback: None,
                },
            },
        ],
        initial: 1,
    }
}

/// Accept exactly this range of tokens and forget their values.
#[inline]
#[must_use]
pub fn any_of<I: Input>(range: Range<I>) -> Deterministic<I> {
    Graph {
        states: vec![
            State {
                transitions: Curry::Scrutinize {
                    filter: RangeMap(BTreeMap::new()),
                    fallback: None,
                },
                non_accepting: BTreeSet::new(),
            },
            State {
                non_accepting: iter::once(format!(
                    "Expected only a single token on [{}..={}] but got another token after it",
                    range.first.to_src(),
                    range.last.to_src(),
                ))
                .collect(),
                transitions: Curry::Scrutinize {
                    filter: RangeMap(
                        iter::once((
                            range,
                            Transition::Lateral {
                                dst: 0,
                                update: None,
                            },
                        ))
                        .collect(),
                    ),
                    fallback: None,
                },
            },
        ],
        initial: 1,
    }
}

/// Accept exactly this token and forget its value.
#[inline]
#[must_use]
pub fn toss<I: Input>(token: I) -> Deterministic<I> {
    any_of(Range::unit(token))
}
