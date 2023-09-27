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
    /// Escape to characters that can unambiguously represent this input as a Rust *identifier*.
    #[must_use]
    fn escape(&self) -> String;
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
    #[inline]
    #[allow(unsafe_code)]
    fn escape(&self) -> String {
        // NOTE: This might technically create ambiguity errors, but it's overwhelmingly unlikely to.
        // If it ever does for you, let me know.
        match *self {
            'A'..='Z' | 'a'..='z' | '0'..='9' => return core::iter::once(self).collect(),
            '\u{1}'..='\u{8}' | '\u{b}' | '\u{e}'..='\u{1f}' | '\u{80}'.. => {
                return format!("_x{:X}_", u32::from(*self))
            }
            '\0' => "_null_",
            '\t' => "_tab_",
            '\n' => "_newline_",
            '\u{c}' => "_newpage_",
            '\r' => "_return_",
            ' ' => "_space_",
            '!' => "_bang_",
            '"' => "_quotes_",
            '#' => "_pound_",
            '$' => "_dollar_",
            '%' => "_percent_",
            '&' => "_ampersand_",
            '\'' => "_apostrophe_",
            '(' => "_lparen_",
            ')' => "_rparen_",
            '*' => "_star_",
            '+' => "_plus_",
            ',' => "_comma_",
            '-' => "_hyphen_",
            '.' => "_dot_",
            '/' => "_slash_",
            ':' => "_colon_",
            ';' => "_semicolon_",
            '<' => "_lessthan_",
            '=' => "_equals_",
            '>' => "_greaterthan_",
            '?' => "_question_",
            '@' => "_at_",
            '[' => "_lbracket_",
            '\\' => "_backslash_",
            ']' => "_rbracket_",
            '^' => "_caret_",
            '_' => "_underscore_",
            '`' => "_backtick_",
            '{' => "_lbrace_",
            '}' => "_rbrace_",
            '|' => "_pipe_",
            '~' => "_tilde_",
            '\u{7f}' => "_delete_",
        }
        .to_owned()
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
    #[inline]
    fn escape(&self) -> String {
        char::from(*self).escape()
    }
}
