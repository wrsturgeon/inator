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

// TODO: minimal path from any node to an initial state (then print this as a doc-comment above each state function)

// TODO (related to the above):
// trait carrying the minimal string as a `const` member,
// plus a type to be defined by the user,
// plus a `const _CHECK: ()` which checks that
// whenever a DFA changes, we don't just
// arbitrarily continue using the wrong node's info,
// and so users don't have to remember node numbers
// Actually, could we make traits generic over `const &'static str`?
// Then nodes could just look up their minimal paths,
// which should all be unique (mental proof but make a bit more sure)
// Basically the idea is that we can't minimze a pushdown automaton
// but any DFA that can format input should be able to reach all relevant states,
// so if we allow the users to "ride" these states,
// they can do the hard work while guaranteeing minimal size

// TODO: move to functions from input to output rather than just inputs
// so e.g. we can write functions that take any uppercase letter instead of manually enumerating A, B, C, ..., Z
// technically Russell's paradox if we allow functions to determine membership but _come on_

pub mod decision;
mod expr;
pub mod format;

pub use expr::Expression;

/// Accept exactly this token.
#[inline(always)]
pub fn a<I: Clone + Ord>(input: I) -> decision::Parser<I> {
    decision::Parser::unit(input)
}

/// When formatting, accept and spit back out this exact token.
#[inline(always)]
pub fn f<I: Clone + Ord>(input: I) -> format::Parser<I> {
    format::Parser::unit(input.clone(), vec![input])
}

/// When formatting, accept this token but replace it with this other stuff.
#[inline(always)]
pub fn r<I: Clone + Ord>(input: I, replace: Vec<I>) -> format::Parser<I> {
    format::Parser::unit(input, replace)
}

/// When formatting, accept this token but delete it.
// TODO: rename to `d`
#[inline(always)]
pub fn opt<I: Clone + Ord>(input: I) -> format::Parser<I> {
    r(input, vec![]).optional()
}

/// When formatting, accept this token but don't spit it out again.
#[inline(always)]
pub fn space() -> format::Parser<char> {
    (f(' ') >> r(' ', vec![]).star()).optional()
}

/// When formatting, accept this token but don't spit it out again.
#[inline(always)]
pub fn no_space() -> format::Parser<char> {
    opt(' ').repeat()
}

/// When formatting, accept this token but don't spit it out again.
#[inline(always)]
pub fn req_space() -> format::Parser<char> {
    f(' ') >> opt(' ').repeat()
}
