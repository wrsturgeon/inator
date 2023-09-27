/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Allowed expressions.

use proc_macro2::Span;

/// Convert to a variety of source-code-related formats.
pub trait Expression: core::fmt::Debug {
    /// Convert `&Self -> syn::Expr`.
    #[must_use]
    fn to_expr(&self) -> syn::Expr;
    /// Convert `&Self -> syn::Pat`.
    #[must_use]
    fn to_pattern(&self) -> syn::Pat;
    /// Write a `syn::Type` type representing this value's type.
    #[must_use]
    fn to_type() -> syn::Type;
}

impl Expression for char {
    #[inline]
    fn to_expr(&self) -> syn::Expr {
        syn::Expr::Lit(syn::ExprLit {
            attrs: vec![],
            lit: syn::Lit::Char(syn::LitChar::new(*self, Span::call_site())),
        })
    }
    #[inline]
    fn to_pattern(&self) -> syn::Pat {
        syn::Pat::Lit(syn::ExprLit {
            attrs: vec![],
            lit: syn::Lit::Char(syn::LitChar::new(*self, Span::call_site())),
        })
    }
    #[inline]
    fn to_type() -> syn::Type {
        syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path {
                leading_colon: None,
                segments: core::iter::once(syn::PathSegment {
                    ident: syn::Ident::new("char", Span::call_site()),
                    arguments: syn::PathArguments::None,
                })
                .collect(),
            },
        })
    }
}

impl Expression for u8 {
    #[inline]
    fn to_expr(&self) -> syn::Expr {
        syn::Expr::Lit(syn::ExprLit {
            attrs: vec![],
            lit: syn::Lit::Byte(syn::LitByte::new(*self, Span::call_site())),
        })
    }
    #[inline]
    fn to_pattern(&self) -> syn::Pat {
        syn::Pat::Lit(syn::ExprLit {
            attrs: vec![],
            lit: syn::Lit::Byte(syn::LitByte::new(*self, Span::call_site())),
        })
    }
    #[inline]
    fn to_type() -> syn::Type {
        syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path {
                leading_colon: None,
                segments: core::iter::once(syn::PathSegment {
                    ident: syn::Ident::new("u8", Span::call_site()),
                    arguments: syn::PathArguments::None,
                })
                .collect(),
            },
        })
    }
}
