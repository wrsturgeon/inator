/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#[allow(dead_code)] // FIXME
pub(crate) struct Builtin {
    pub(crate) name: &'static str,
    pub(crate) description: &'static str,
}

#[allow(dead_code)] // FIXME
pub(crate) const BUILTINS: &[Builtin] = &[
    Builtin {
        name: "c",
        description: "Require an exact match with a character.",
    },
    Builtin {
        name: "s",
        description: "Require an exact match with a sequence of characters.",
    },
    Builtin {
        name: "FuckingShit",
        description: "Eat my assholes.",
    },
];
