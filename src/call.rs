/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! User-defined functions to call on transitions.

use crate::dfa::config_path;
use core::iter::once;
use proc_macro2::{Ident, Span};
use syn::{token::Paren, Expr, ExprCall, ExprPath, Path, PathArguments, PathSegment};

#[cfg(feature = "quickcheck")]
use quickcheck::*;

/// A user-defined function to call on a transition.
/// Uses `String` instead of `&str` for property-testing
#[allow(dead_code)] // <-- FIXME
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum Call {
    /// Ignore this token, but wait to call a function, passing on any arguments so far.
    #[default]
    Pass,
    /// Read this token onto the stack but don't call yet.
    Stash,
    /// Read this token and immediately call a function on it, which becomes an identifier only AFTER `build.rs`.
    WithToken(String),
    /// Ignore this token and immediately call a function, which becomes an identifier only AFTER `build.rs`.
    WithoutToken(String),
}

#[allow(clippy::multiple_inherent_impl)]
impl Call {
    /// Pretty-print for insertion into a plain English sentence.
    #[inline]
    pub(crate) fn verbal(&self) -> String {
        match *self {
            Self::Pass => "discarding the token".to_owned(),
            Self::Stash => "stashing the token for later".to_owned(),
            Self::WithToken(ref name) | Self::WithoutToken(ref name) => format!("calling `{name}`"),
        }
    }

    /// Remove all calls (set them to `None`).
    #[must_use]
    #[cfg(test)]
    #[inline(always)]
    #[allow(clippy::unused_self)]
    pub(crate) fn remove_calls(self) -> Self {
        Self::Pass
    }

    /// Check if two calls are compatible and can be reduced or postponed.
    #[inline]
    pub(crate) fn compat(self, other: Self) -> Option<Result<Self, (bool, Self, Self)>> {
        match (self, other) {
            (Self::Pass, Self::Pass) => Some(Ok(Self::Pass)),
            (Self::Pass | Self::Stash, Self::Pass | Self::Stash) => Some(Ok(Self::Stash)),
            (Self::WithToken(lhs), Self::WithToken(rhs)) if lhs == rhs => {
                Some(Ok(Self::WithToken(lhs)))
            }
            (Self::WithoutToken(lhs), Self::WithoutToken(rhs)) if lhs == rhs => {
                Some(Ok(Self::WithoutToken(lhs)))
            }
            // Rust forbids identically named functions with different arguments
            (Self::WithToken(_), Self::WithoutToken(_))
            | (Self::WithoutToken(_), Self::WithToken(_)) => None,
            (lhs @ Self::WithToken(_), Self::Pass | Self::Stash) => {
                Some(Err((true, lhs, Self::Pass)))
            }
            (lhs @ Self::WithoutToken(_), Self::Pass | Self::Stash) => {
                Some(Err((false, lhs, Self::Pass)))
            }
            (Self::Pass | Self::Stash, rhs @ (Self::WithToken(_) | Self::WithoutToken(_))) => {
                Some(Err((false, Self::Pass, rhs)))
            }
            (lhs @ Self::WithToken(_), rhs @ Self::WithToken(_)) => Some(Err((true, lhs, rhs))),
            (lhs @ Self::WithoutToken(_), rhs @ Self::WithoutToken(_)) => {
                Some(Err((false, lhs, rhs)))
            }
        }
    }

    /// Convert to a `syn::Expr`.
    #[inline]
    pub(crate) fn to_expr(&self) -> Expr {
        match *self {
            Self::Pass => Expr::Path(ExprPath {
                attrs: vec![],
                qself: None,
                path: Path {
                    leading_colon: None,
                    segments: once(PathSegment {
                        ident: Ident::new("acc", Span::call_site()),
                        arguments: PathArguments::None,
                    })
                    .collect(),
                },
            }),
            Self::Stash => Expr::Path(ExprPath {
                attrs: vec![],
                qself: None,
                path: Path {
                    leading_colon: None,
                    segments: once(PathSegment {
                        ident: Ident::new("TODO_STASH", Span::call_site()),
                        arguments: PathArguments::None,
                    })
                    .collect(),
                },
            }),
            Self::WithToken(ref name) => Expr::Call(ExprCall {
                attrs: vec![],
                func: Box::new(Expr::Path(ExprPath {
                    attrs: vec![],
                    qself: None,
                    path: config_path(name, name),
                })),
                paren_token: Paren::default(),
                args: [
                    Expr::Path(ExprPath {
                        attrs: vec![],
                        qself: None,
                        path: Path {
                            leading_colon: None,
                            segments: once(PathSegment {
                                ident: Ident::new("acc", Span::call_site()),
                                arguments: PathArguments::None,
                            })
                            .collect(),
                        },
                    }),
                    Expr::Path(ExprPath {
                        attrs: vec![],
                        qself: None,
                        path: Path {
                            leading_colon: None,
                            segments: once(PathSegment {
                                ident: Ident::new("token", Span::call_site()),
                                arguments: PathArguments::None,
                            })
                            .collect(),
                        },
                    }),
                ]
                .into_iter()
                .collect(),
            }),
            Self::WithoutToken(ref name) => Expr::Call(ExprCall {
                attrs: vec![],
                func: Box::new(Expr::Path(ExprPath {
                    attrs: vec![],
                    qself: None,
                    path: config_path(name, name),
                })),
                paren_token: Paren::default(),
                args: once(Expr::Path(ExprPath {
                    attrs: vec![],
                    qself: None,
                    path: Path {
                        leading_colon: None,
                        segments: once(PathSegment {
                            ident: Ident::new("acc", Span::call_site()),
                            arguments: PathArguments::None,
                        })
                        .collect(),
                    },
                }))
                .collect(),
            }),
        }
    }
}

#[cfg(feature = "quickcheck")]
impl Arbitrary for Call {
    #[inline]
    #[allow(clippy::as_conversions, clippy::unwrap_used, trivial_casts)]
    fn arbitrary(g: &mut Gen) -> Self {
        g.choose(&[
            (|_| Self::Pass) as fn(_) -> _,
            |_| Self::Stash,
            |r| Self::WithToken(Arbitrary::arbitrary(r)),
            |r| Self::WithoutToken(Arbitrary::arbitrary(r)),
        ])
        .unwrap()(g)
    }

    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        use core::iter::*;

        match *self {
            Self::Pass => Box::new(empty()),
            Self::Stash => Box::new(once(Self::Pass)),
            Self::WithToken(ref name) => Box::new(
                Self::Stash
                    .shrink()
                    .chain(name.shrink().map(Self::WithToken)),
            ),
            Self::WithoutToken(ref name) => Box::new(
                Self::WithToken(name.clone())
                    .shrink()
                    .chain(name.shrink().map(Self::WithoutToken)),
            ),
        }
    }
}
