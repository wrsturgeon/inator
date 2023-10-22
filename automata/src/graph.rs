/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Automaton loosely based on visibly pushdown automata.

use crate::{Check, Ctrl, IllFormed, Input, Output, ParseError, Stack, State};
use core::num::NonZeroUsize;
use std::{collections::BTreeSet, ffi::OsStr, fs, io, path::Path, process::Command};

/// One token corresponds to at most one transition.
pub type Deterministic<I, S, O> = Graph<I, S, O, usize>;

/// One token corresponds to as many transitions as it would like;
/// if any of these transitions eventually accept, the whole thing accepts.
pub type Nondeterministic<I, S, O> = Graph<I, S, O, BTreeSet<usize>>;

/// Automaton loosely based on visibly pushdown automata.
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Graph<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> {
    /// Every state, indexed.
    pub states: Vec<State<I, S, O, C>>,
    /// Initial state of the machine (before reading input).
    pub initial: C,
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Graph<I, S, O, C> {
    /// Check well-formedness.
    /// # Errors
    /// When not well-formed (with a witness).
    #[inline]
    pub fn check(&self) -> Result<(), IllFormed<I, S, O, C>> {
        let n_states = self.states.len();
        if let Some(i) = self
            .initial
            .view()
            .fold(None, |acc, i| acc.or_else(|| (i >= n_states).then_some(i)))
        {
            return Err(IllFormed::OutOfBounds(i));
        }
        NonZeroUsize::new(n_states).map_or(Ok(()), |nz| {
            self.states.iter().try_fold((), |(), state| state.check(nz))
        })
    }

    /// Run this parser to completion and check the result.
    /// # Errors
    /// If the parser determines there should be an error.
    #[inline]
    #[allow(unsafe_code)]
    pub fn accept<In: IntoIterator<Item = I>>(
        &self,
        input: In,
    ) -> Result<Option<O>, ParseError<I, S, O, C>> {
        use crate::Run;
        let mut run = input.run(self);
        for r in &mut run {
            drop(r?);
        }
        let output = run
            .ctrl
            .view()
            .any(|i| get!(self.states, i).accepting)
            .then(|| {
                // SAFETY: Never uninitialized except inside one function (and initialized before it exits).
                unsafe { run.output.assume_init() }
            });
        Ok(output)
    }
}

impl<I: Input, S: Stack, O: Output> Graph<I, S, O, usize> {
    /// Write this parser as a Rust source file.
    /// # Errors
    /// If file creation or formatting fails.
    #[inline]
    pub fn to_file<P: AsRef<OsStr> + AsRef<Path>>(&self, path: P) -> io::Result<()> {
        fs::write(&path, self.to_src())?;
        Command::new("rustfmt").arg(path).output().map(|_| {})
    }
}
