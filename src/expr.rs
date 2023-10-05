/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Allowed expressions.

use core::{fmt::Debug, iter::once};
use proc_macro2::Span;
use syn::{
    Expr, ExprLit, Lit, LitByte, LitChar, Pat, Path, PathArguments, PathSegment, Type, TypePath,
};

/// Convert to a variety of source-code-related formats.
pub trait Expression: Debug {
    /// Convert `&Self -> syn::Expr`.
    #[must_use]
    fn to_expr(&self) -> Expr;
    /// Convert `&Self -> syn::Pat`.
    #[must_use]
    fn to_pattern(&self) -> Pat;
    /// Write a `syn::Type` type representing this value's type.
    #[must_use]
    fn to_type() -> Type;
}

impl Expression for char {
    #[inline]
    fn to_expr(&self) -> Expr {
        Expr::Lit(ExprLit {
            attrs: vec![],
            lit: Lit::Char(LitChar::new(*self, Span::call_site())),
        })
    }
    #[inline]
    fn to_pattern(&self) -> Pat {
        Pat::Lit(ExprLit {
            attrs: vec![],
            lit: Lit::Char(LitChar::new(*self, Span::call_site())),
        })
    }
    #[inline]
    fn to_type() -> Type {
        Type::Path(TypePath {
            qself: None,
            path: Path {
                leading_colon: None,
                segments: once(PathSegment {
                    ident: syn::Ident::new("char", Span::call_site()),
                    arguments: PathArguments::None,
                })
                .collect(),
            },
        })
    }
}

impl Expression for u8 {
    #[inline]
    fn to_expr(&self) -> Expr {
        Expr::Lit(ExprLit {
            attrs: vec![],
            lit: Lit::Byte(LitByte::new(*self, Span::call_site())),
        })
    }
    #[inline]
    fn to_pattern(&self) -> Pat {
        Pat::Lit(ExprLit {
            attrs: vec![],
            lit: Lit::Byte(LitByte::new(*self, Span::call_site())),
        })
    }
    #[inline]
    fn to_type() -> Type {
        Type::Path(TypePath {
            qself: None,
            path: Path {
                leading_colon: None,
                segments: once(PathSegment {
                    ident: syn::Ident::new("u8", Span::call_site()),
                    arguments: PathArguments::None,
                })
                .collect(),
            },
        })
    }
}
