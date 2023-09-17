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
#![allow(unused_crate_dependencies)] // <-- FIXME: remove

#[allow(unused_extern_crates)]
extern crate proc_macro;

mod builtins;

use inator_automata::{Expression, Nfa};
use proc_macro2::{Ident, Span, TokenStream};
use syn::{__private::ToTokens, spanned::Spanned, Token};

/// Define a parser.
#[proc_macro_attribute]
pub fn inator(
    attr: proc_macro::TokenStream,
    source: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    fallible(&attr.into(), source.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Any processing that could fail.
#[inline(always)] // one call site
fn fallible(attr: &TokenStream, source: TokenStream) -> syn::Result<TokenStream> {
    check_no_macro_args(attr)?;
    process_fn_source(source)
}

/// Make sure the macro wasn't called with parenthesized arguments.
#[inline(always)] // one call site
fn check_no_macro_args(attr: &TokenStream) -> syn::Result<()> {
    if attr.is_empty() {
        Ok(())
    } else {
        Err(syn::Error::new(
            attr.span(),
            "Expected no arguments to this macro",
        ))
    }
}

/// Read the source of the function the user wrote.
#[inline(always)] // one call site
#[allow(unreachable_code)] // <-- FIXME
fn process_fn_source(source: TokenStream) -> syn::Result<TokenStream> {
    // Make sure we have a function, not something else
    #[allow(clippy::wildcard_enum_match_arm)]
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
    let (in_t, out_t) = standardize_trait(rtn_t)?;
    for arg in &mut f.sig.inputs {
        let syn::FnArg::Typed(ref mut t) = *arg else {
            return Err(syn::Error::new(
                arg.span(),
                "Parsers can't take `self` arguments",
            ));
        };
        drop(standardize_trait(&mut t.ty)); // Ignore non-`Parse` types while fixing `Parse` types
    }

    // Start the output `TokenStream` with a private module for DFA states
    Ok(compile(f, in_t, out_t)?.into_token_stream())
}

/// Write a new set of functions based on the source function.
#[inline]
fn compile(mut f: syn::ItemFn, in_t: syn::Type, out_t: syn::Type) -> syn::Result<TokenStream> {
    let id = f.sig.ident.to_string();
    let mut module_contents = vec![syn::Item::Use(syn::ItemUse {
        attrs: vec![],
        vis: syn::Visibility::Inherited,
        use_token: Token!(use)(Span::call_site()),
        leading_colon: None,
        tree: syn::UseTree::Group(syn::UseGroup {
            brace_token: syn::token::Brace::default(),
            items: [
                syn::UseTree::Path(syn::UsePath {
                    ident: Ident::new("super", Span::call_site()),
                    colon2_token: Token!(::)(Span::call_site()),
                    tree: Box::new(syn::UseTree::Glob(syn::UseGlob {
                        star_token: Token!(*)(Span::call_site()),
                    })),
                }),
                // syn::UseTree::Path(syn::UsePath {
                //     ident: Ident::new("inator", Span::call_site()),
                //     colon2_token: Token!(::)(Span::call_site()),
                //     tree: Box::new(syn::UseTree::Path(syn::UsePath {
                //         ident: Ident::new("prelude", Span::call_site()),
                //         colon2_token: Token!(::)(Span::call_site()),
                //         tree: Box::new(syn::UseTree::Glob(syn::UseGlob {
                //             star_token: Token!(*)(Span::call_site()),
                //         })),
                //     })),
                // }),
            ]
            .into_iter()
            .collect(),
        }),
        semi_token: Token!(;)(Span::call_site()),
    })];
    let (init_fn, state_fns) = process_fn_body(&f.block)?
        .minimize()
        .as_source(&f, in_t, out_t);
    f.block.stmts.insert(
        0, /* ouch */
        syn::Stmt::Item(syn::Item::Use(syn::ItemUse {
            attrs: vec![],
            vis: syn::Visibility::Inherited,
            use_token: Token!(use)(Span::call_site()),
            leading_colon: None,
            tree: syn::UseTree::Path(syn::UsePath {
                ident: Ident::new("inator", Span::call_site()),
                colon2_token: Token!(::)(Span::call_site()),
                tree: Box::new(syn::UseTree::Path(syn::UsePath {
                    ident: Ident::new("mirage", Span::call_site()),
                    colon2_token: Token!(::)(Span::call_site()),
                    tree: Box::new(syn::UseTree::Glob(syn::UseGlob {
                        star_token: Token!(*)(Span::call_site()),
                    })),
                })),
            }),
            semi_token: Token!(;)(Span::call_site()),
        })),
    );
    // Copy the original so that IDEs don't think the body has been deleted:
    module_contents.push(syn::Item::Fn(f));
    module_contents.extend(state_fns);
    let mut ts = init_fn.into_token_stream();
    #[allow(clippy::arithmetic_side_effects)]
    syn::ItemMod {
        attrs: vec![],
        vis: syn::Visibility::Inherited,
        unsafety: None,
        mod_token: Token!(mod)(Span::call_site()),
        ident: Ident::new(&("_inator_automaton_".to_owned() + &id), Span::call_site()),
        content: Some((syn::token::Brace::default(), module_contents)),
        semi: None,
    }
    .to_tokens(&mut ts);
    Ok(ts)
}

/// Standardize `Parse(I) -> O` to `Parse<I, Output = O>` for all I, O
#[inline]
fn standardize_trait(rtn_t: &mut syn::Type) -> syn::Result<(syn::Type, syn::Type)> {
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
    Ok(match *arguments {
        syn::PathArguments::Parenthesized(ref mut args) => {
            let Some(first_arg) = args.inputs.pop() else {
                return Err(syn::Error::new(args.span(), "Expected an input type"));
            };
            let in_t = first_arg.into_value();
            if let Some(next_input_arg) = args.inputs.pop() {
                return Err(syn::Error::new(
                    next_input_arg.span(),
                    "Expected a single input type",
                ));
            };
            let mut p: syn::punctuated::Punctuated<_, _> =
                core::iter::once(syn::GenericArgument::Type(in_t.clone())).collect();
            let out_t = match args.output {
                syn::ReturnType::Type(_, ref out_t) => syn::Type::clone(out_t),
                syn::ReturnType::Default => syn::Type::Tuple(syn::TypeTuple {
                    paren_token: syn::token::Paren::default(),
                    elems: syn::punctuated::Punctuated::new(),
                }),
            };
            p.push(syn::GenericArgument::AssocType(syn::AssocType {
                ident: Ident::new("Output", Span::call_site()),
                generics: None,
                eq_token: Token!(=)(Span::call_site()),
                ty: syn::Type::clone(&out_t),
            }));
            *arguments = syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                colon2_token: None,
                lt_token: Token!(<)(Span::call_site()),
                args: p,
                gt_token: Token!(>)(Span::call_site()),
            });
            (in_t, out_t)
        }
        syn::PathArguments::AngleBracketed(_) => {
            todo!()
        }
        syn::PathArguments::None => {
            return Err(syn::Error::new(
                arguments.span(),
                "Expected generic arguments formatted either like\
                    `Parse(char) -> char` or `Parse<char, Output = char>`",
            ))
        }
    })
}

#[inline]
fn process_fn_body(body: &syn::Block) -> syn::Result<Nfa<Expression>> {
    if body.stmts.len() > 1 {
        return Err(syn::Error::new(
            body.span(),
            "Parser functions should be \
            one statement long \
            (an expression without a semicolon)",
        ));
    }
    let Some(statement) = body.stmts.first() else {
        return Err(syn::Error::new(
            body.span(),
            "Provide an expression that constructs a parser \
            (e.g. `{ p!('A' | 'B') >> c }`)",
        ));
    };
    let expr = match *statement {
        syn::Stmt::Local(ref local) => {
            return Err(syn::Error::new(
                local.span(),
                "Let-expressions not yet supported in `#[inator]` functions",
            ))
        }
        syn::Stmt::Item(ref item) => {
            return Err(syn::Error::new(
                item.span(),
                "Item expressions not yet supported in `#[inator]` functions",
            ))
        }
        syn::Stmt::Expr(ref expr, None) => expr,
        syn::Stmt::Expr(_, Some(semi)) => {
            return Err(syn::Error::new(
                semi.span(),
                "Remove the semicolon to return this result",
            ))
        }
        syn::Stmt::Macro(ref mac) => {
            return Err(syn::Error::new(
                mac.span(),
                "Statement-position macros not yet supported in `#[inator]` functions",
            ))
        }
    };
    process_expression(expr)
}

#[inline]
fn process_expression(expr: &syn::Expr) -> syn::Result<Nfa<Expression>> {
    if let syn::Expr::Macro(syn::ExprMacro { ref mac, .. }) = *expr {
        if mac.path.leading_colon.is_some()
            || mac.path.segments.len() != 1
            || mac.path.segments.first().unwrap().ident != "p"
        {
            return Err(syn::Error::new(
                mac.path.span(),
                "`p!(...)` is the only macro allowed in `#[inator]` functions",
            ));
        }
        // TODO: Why the fuck doesn't `syn::Pat` implement `syn::Parse`?
        process_non_macro(&syn::parse2(mac.tokens.clone())?, false)
    } else {
        process_non_macro(expr, true)
    }
}

#[inline]
#[allow(unused_variables)] // <-- FIXME
fn process_non_macro(expr: &syn::Expr, outside_p: bool) -> syn::Result<Nfa<Expression>> {
    Ok(match *expr {
        syn::Expr::Binary(ref bin) => match bin.op {
            syn::BinOp::BitOr(_) => {
                process_expression(&bin.left)? | process_expression(&bin.right)?
            }
            syn::BinOp::BitAnd(_) if outside_p => {
                process_expression(&bin.left)? & process_expression(&bin.right)?
            }
            syn::BinOp::Shl(_) if outside_p => {
                process_expression(&bin.left)? << process_expression(&bin.right)?
            }
            syn::BinOp::Shr(_) if outside_p => {
                process_expression(&bin.left)? >> process_expression(&bin.right)?
            }
            op => {
                return Err(syn::Error::new(
                    op.span(),
                    "Only the `|` operator \
                                (meaning \"any one of these\" in a Rust pattern) \
                                is supported as an infix operator in `p!(...)`",
                ))
            }
        },
        syn::Expr::Infer(_) => panic!("infer"),
        syn::Expr::Lit(ref lit) => Nfa::unit(match lit.lit {
            syn::Lit::Str(ref s) => Expression::String(s.value()),
            syn::Lit::Char(ref c) => Expression::Char(c.value()),
            syn::Lit::ByteStr(ref bs) => Expression::ByteString(bs.value()),
            syn::Lit::Int(ref i) => Expression::Int(i.base10_digits().bytes().collect()),
            syn::Lit::Float(_) => {
                return Err(syn::Error::new(
                    lit.lit.span(),
                    "Floating-point literals not supported (they aren't `Ord`)",
                ))
            }
            syn::Lit::Bool(ref b) => Expression::Bool(b.value),
            syn::Lit::Verbatim(_) | _ => {
                return Err(syn::Error::new(
                    lit.lit.span(),
                    "Couldn't parse this literal as a \
                    string, character, byte string, integer, or boolean",
                ))
            }
        }),
        syn::Expr::Paren(ref paren) => {
            return Err(syn::Error::new(paren.span(), "Unnecessary parentheses"))
        }
        syn::Expr::Path(ref path) => panic!("path"),
        syn::Expr::Range(ref range) => panic!("range"),
        syn::Expr::Reference(ref rf) => panic!("reference"),
        syn::Expr::Struct(ref st) => panic!("struct"),
        syn::Expr::Tuple(ref tuple) => panic!("tuple"),
        syn::Expr::Unary(ref unary) => panic!("unary"),
        syn::Expr::Verbatim(ref verbatim) => panic!("verbatim"),
        _ => {
            return Err(syn::Error::new(
                expr.span(),
                "This type of expression is not permitted in `#[inator]` functions",
            ))
        }
    })
}
