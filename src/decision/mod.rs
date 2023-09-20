/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Automata that can only decide whether an input is in a given language.

mod brzozowski;
pub(crate) mod dfa;
mod fuzz;
pub(crate) mod nfa;
mod ops;
mod powerset_construction;

pub use {
    dfa::Graph as Dfa,
    fuzz::{Fuzzer, NeverAccepts},
    nfa::Graph as Nfa,
};
