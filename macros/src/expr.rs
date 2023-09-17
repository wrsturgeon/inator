/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Allowed expressions.

#[allow(dead_code)] // <-- FIXME
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum Expression {
    /// Boolean.
    Bool(bool),
    /// Byte.
    Byte(u8),
    /// Character.
    Char(char),
    /// Integer.
    Int(isize),
    // /// Floating-point number.
    // Float(f64),
    /// Byte-string.
    ByteString(&'static [u8]),
    /// String.
    String(String),
    /// Uninterpretable.
    Verbatim(core::convert::Infallible),
}
