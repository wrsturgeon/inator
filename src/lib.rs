/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Nondeterministic finite automata with epsilon transitions and algorithms to compile them into optimal deterministic finite automata.

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
        #[allow(unsafe_code)]
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
        #[allow(unsafe_code)]
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
        #[allow(unsafe_code)]
        let result = unsafe { $expr.get_unchecked_mut($index) };
        result
    }};
}

// TODO: write an inherent impl for `Nfa<char>` with a bunch of stuff like `parenthesized`

// TODO: have a recommended path for each thing, e.g. instead of `optional` have `encouraged` and `discouraged` then use this to format

mod brzozowski;
mod dfa;
mod expr;
mod fuzz;
mod nfa;
mod ops;
mod powerset_construction;

#[cfg(test)]
mod test;

pub use {
    dfa::Graph as Compiled,
    expr::Expression,
    fuzz::{Fuzzer, NeverAccepts},
    ops::{Lazy as Parser, StillPostponed},
};

use ops::Lazy;

/// Promise a parser that can't be computed yet (usually because it nests itself).
#[must_use]
#[inline(always)]
#[allow(clippy::ref_option_ref)]
pub const fn postponed<I: Clone + Ord>() -> Lazy<I> {
    Lazy::Postponed
}

/// Accept only the empty string.
#[must_use]
#[inline(always)]
pub fn empty<I: Clone + Ord>() -> Lazy<I> {
    Lazy::Immediate(nfa::Graph::empty())
}

/// Accept this token if we see it here, but throw it away.
#[must_use]
#[inline(always)]
pub fn ignore<I: Clone + Ord>(token: I) -> Lazy<I> {
    Lazy::Immediate(nfa::Graph::unit(token, None))
}

/// Accept this token if we see it here, then call this user-defined function on it.
#[must_use]
#[inline(always)]
pub fn on<I: Clone + Ord>(token: I, fn_name: &'static str) -> Lazy<I> {
    Lazy::Immediate(nfa::Graph::unit(token, Some(fn_name)))
}

/// Accept this sequence of tokens if we see it here, then call this user-defined function on it.
#[must_use]
#[inline(always)]
#[allow(clippy::arithmetic_side_effects)]
pub fn on_seq<I: Clone + Ord, II: IntoIterator<Item = I>>(
    tokens: II,
    fn_name: &'static str,
) -> Lazy<I> {
    let mut v: Vec<_> = tokens.into_iter().collect();
    let Some(last) = v.pop() else {
        return empty();
    };
    seq(v.into_iter().map(|token| ignore(token))) >> on(last, fn_name)
}

/// Accept either this token or nothing.
#[inline]
#[must_use]
pub fn opt<I: Clone + Ord>(token: I) -> Lazy<I> {
    ignore(token).optional()
}

/// A single character of whitespace (or exactly one "\r\n").
#[inline]
#[must_use]
#[allow(clippy::arithmetic_side_effects)]
pub fn single_space() -> Lazy<char> {
    ignore(' ') | ignore('\n') | (ignore('\r') >> ignore('\n'))
}

/// Any amount of whitespace.
#[inline]
#[must_use]
#[allow(clippy::arithmetic_side_effects)]
pub fn space() -> Lazy<char> {
    single_space().star()
}

/// Surround this language in parentheses.
/// Note that whitespace around the language--e.g. "( A )"--is fine.
#[inline]
#[must_use]
#[allow(clippy::arithmetic_side_effects, clippy::needless_lifetimes)]
pub fn parenthesized(p: Lazy<char>) -> Lazy<char> {
    ignore('(') + p + ignore(')')
}

/// Accept anything accepted by any of these parsers.
#[inline]
#[must_use]
pub fn any<I: Clone + Ord, II: IntoIterator<Item = Lazy<I>>>(alternatives: II) -> Lazy<I> {
    alternatives
        .into_iter()
        .fold(Lazy::Immediate(nfa::Graph::void()), |acc, p| acc | p)
}

/// Accept if and only if each parser accepts in order.
#[inline]
#[must_use]
#[allow(clippy::arithmetic_side_effects)]
pub fn seq<I: Clone + Ord, II: IntoIterator<Item = Lazy<I>>>(alternatives: II) -> Lazy<I> {
    alternatives
        .into_iter()
        .fold(Lazy::Immediate(nfa::Graph::void()), |acc, p| acc >> p)
}
