/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Allowed expressions.

use proc_macro2::Span;

/// Any possible expression to be matched against.
#[non_exhaustive]
#[allow(dead_code)] // <-- FIXME
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Expression {
    /// Boolean.
    Bool(bool),
    /// Byte.
    Byte(u8),
    /// Character.
    Char(char),
    /// Integer (individual digits).
    Int(Vec<u8>),
    // /// Floating-point number.
    // Float(f64),
    /// Byte-string.
    ByteString(Vec<u8>),
    /// String.
    String(String),
}

impl Expression {
    /// Convert to a `syn::Pat`.
    #[inline]
    #[must_use]
    pub fn as_pattern(&self) -> syn::Pat {
        match *self {
            Self::Bool(b) => syn::Pat::Lit(syn::ExprLit {
                attrs: vec![],
                lit: syn::Lit::Bool(syn::LitBool::new(b, Span::call_site())),
            }),
            Self::Byte(b) => syn::Pat::Lit(syn::ExprLit {
                attrs: vec![],
                lit: syn::Lit::Byte(syn::LitByte::new(b, Span::call_site())),
            }),
            Self::Char(c) => syn::Pat::Lit(syn::ExprLit {
                attrs: vec![],
                lit: syn::Lit::Char(syn::LitChar::new(c, Span::call_site())),
            }),
            Self::Int(ref i) => syn::Pat::Lit(syn::ExprLit {
                attrs: vec![],
                lit: syn::Lit::Int(syn::LitInt::new(
                    #[allow(unsafe_code)]
                    // SAFETY: Untouched from `syn`
                    unsafe {
                        core::str::from_utf8_unchecked(i)
                    },
                    Span::call_site(),
                )),
            }),
            Self::ByteString(ref bs) => syn::Pat::Lit(syn::ExprLit {
                attrs: vec![],
                lit: syn::Lit::ByteStr(syn::LitByteStr::new(bs, Span::call_site())),
            }),
            Self::String(ref s) => syn::Pat::Lit(syn::ExprLit {
                attrs: vec![],
                lit: syn::Lit::Str(syn::LitStr::new(s, Span::call_site())),
            }),
        }
    }
}
