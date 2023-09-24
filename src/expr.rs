/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Allowed expressions.

use proc_macro2::Span;

/// Convert to a variety of source-code-related formats.
pub trait Expression {
    /// Convert `&Self -> inator::Expression`.
    #[must_use]
    fn to_pattern(&self) -> syn::Pat;
    /// Write a string that looks like Rust source (for pretty-printing only).
    #[must_use]
    fn to_source(&self) -> String;
    /// Write a `syn` type representing this type.
    #[must_use]
    fn to_type() -> syn::Type;
}

impl Expression for char {
    #[inline(always)]
    fn to_pattern(&self) -> syn::Pat {
        syn::Pat::Lit(syn::ExprLit {
            attrs: vec![],
            lit: syn::Lit::Char(syn::LitChar::new(*self, Span::call_site())),
        })
    }
    #[inline]
    fn to_source(&self) -> String {
        format!("'{}'", self.escape_default())
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
    #[inline(always)]
    fn to_pattern(&self) -> syn::Pat {
        syn::Pat::Lit(syn::ExprLit {
            attrs: vec![],
            lit: syn::Lit::Byte(syn::LitByte::new(*self, Span::call_site())),
        })
    }
    #[inline]
    fn to_source(&self) -> String {
        format!("b'{}'", char::from(*self).escape_default())
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
