/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Procedural macros for the `inator` crate.

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

use proc_macro2::{Ident, Span, TokenStream};
use syn::{__private::ToTokens, spanned::Spanned, Token};

#[allow(unused_extern_crates)]
extern crate proc_macro;

mod builtins;

/// Define a parser.
#[proc_macro_attribute]
pub fn inator(
    attr: proc_macro::TokenStream,
    source: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    fallible(attr.into(), source.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[inline(always)] // one call site
fn fallible(attr: TokenStream, source: TokenStream) -> syn::Result<TokenStream> {
    check_no_macro_args(attr)?;
    process_fn_source(source)
}

#[inline(always)] // one call site
fn check_no_macro_args(attr: TokenStream) -> syn::Result<()> {
    if !attr.is_empty() {
        Err(syn::Error::new(
            attr.span(),
            "Expected no arguments to this macro",
        ))
    } else {
        Ok(())
    }
}

#[inline(always)] // one call site
#[allow(unreachable_code)] // <-- FIXME
fn process_fn_source(source: TokenStream) -> syn::Result<TokenStream> {
    // Make sure we have a function, not something else
    let mut f = match syn::parse2(source)? {
        syn::Item::Fn(f) => f,
        other => {
            return Err(syn::Error::new(
                other.span(),
                "Expected a function like `fn p() -> impl Parse(char) -> char { ...`",
            ))
        }
    };

    // Make sure it has a return type (not implicitly `()`)
    let syn::ReturnType::Type(_, ref mut rtn_t) = f.sig.output else {
        return Err(syn::Error::new(
            f.sig.output.span(),
            "Expected a return type like `-> impl Parse(char) -> char`",
        ));
    };

    // Rewrite `Parse(I) -> O` to `Parse<I, Output = O>` for all I, O
    standardize_trait(rtn_t)?;
    for arg in &mut f.sig.inputs {
        let syn::FnArg::Typed(t) = arg else {
            return Err(syn::Error::new(
                arg.span(),
                "Parsers can't take `self` arguments",
            ));
        };
        standardize_trait(&mut t.ty).unwrap_or(()); // Ignore non-`Parse` types while fixing `Parse` types
    }

    // Start the output `TokenStream` with a private module for DFA states
    Ok(compile(f).into_token_stream())
}

#[inline]
fn compile(f: syn::ItemFn) -> TokenStream {
    let v = vec![syn::Item::Fn(f)]; // Copy the original so that IDEs don't think the body has been deleted
    let ts = todo!().into_token_stream();
    syn::ItemMod {
        attrs: vec![],
        vis: syn::Visibility::Inherited,
        unsafety: None,
        mod_token: Token!(mod)(Span::call_site()),
        ident: Ident::new(
            &("_states_".to_owned() + &f.sig.ident.to_string()),
            Span::call_site(),
        ),
        content: Some((syn::token::Brace::default(), v)),
        semi: None,
    }
    .to_tokens(&mut ts);
    ts
}

#[inline]
fn standardize_trait(rtn_t: &mut syn::Type) -> syn::Result<()> {
    // Make sure the return type is `impl ...`
    let syn::Type::ImplTrait(ref mut impl_t) = *rtn_t else {
        return Err(syn::Error::new(
            rtn_t.span(),
            "Expected a return type literally using `impl Parse...`, like `-> impl Parse(char) -> char`",
        ));
    };

    // Make sure we have exactly one bound
    let bound = {
        let mut bounds_iter = impl_t.bounds.iter_mut();
        let Some(first) = bounds_iter.next() else {
            return Err(syn::Error::new(
            impl_t.bounds.span(),
            "Expected a return type literally using `impl Parse...`, like `-> impl Parse(char) -> char`",
        ));
        };
        if let Some(second) = bounds_iter.next() {
            return Err(syn::Error::new(
                second.span(),
                "Expected exactly one trait (`Parse`) but got many",
            ));
        }
        first
    };

    // Make sure that bound is a trait bound (not, e.g., a lifetime)
    let &mut syn::TypeParamBound::Trait(ref mut trait_impl) = bound else {
        return Err(syn::Error::new(
            bound.span(),
            "Expected a return type literally using `impl Parse...`, like `-> impl Parse(char) -> char`",
        ));
    };

    // Make sure that trait is actually `Parse`
    let mut segments = trait_impl.path.segments.iter_mut();
    let Some(&mut syn::PathSegment {
        ref mut ident,
        ref mut arguments,
    }) = segments.next()
    else {
        return Err(syn::Error::new(trait_impl.span(), "Expected a trait"));
    };
    if *ident != "Parse" {
        return Err(syn::Error::new(ident.span(), "Expected the `Parse` trait"));
    }

    // Rewrite to angle brackets if they were parenthesized
    match *arguments {
        syn::PathArguments::Parenthesized(ref mut args) => {
            *arguments = syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                colon2_token: None,
                lt_token: Token!(<)(Span::call_site()),
                args: {
                    let mut p: syn::punctuated::Punctuated<_, _> = args
                        .inputs
                        .iter()
                        .map(|t| syn::GenericArgument::Type(t.clone()))
                        .collect();
                    if let syn::ReturnType::Type(_, ref t) = args.output {
                        p.push(syn::GenericArgument::AssocType(syn::AssocType {
                            ident: Ident::new("Output", Span::call_site()),
                            generics: None,
                            eq_token: Token!(=)(Span::call_site()),
                            ty: syn::Type::clone(t),
                        }));
                    }
                    p
                },
                gt_token: Token!(>)(Span::call_site()),
            })
        }
        syn::PathArguments::AngleBracketed(_) => {}
        syn::PathArguments::None => {
            return Err(syn::Error::new(
                arguments.span(),
                "Expected generic arguments formatted either like\
                    `Parse(char) -> char` or `Parse<char, Output = char>`",
            ))
        }
    }

    Ok(())
}
