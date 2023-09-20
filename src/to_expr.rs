/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Convert to a `syn::Expr`.

/// Convert to a `syn::Expr`.
pub trait ToExpr {
    /// Convert to a `syn::Expr`.
    #[must_use]
    fn to_expr(&self) -> syn::Expr;
}
