/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Execute an automaton on an input sequence.

use crate::{try_merge, Ctrl, Graph, IllFormed, Input, ToSrc};
use core::fmt;

/// Execute an automaton on an input sequence.
#[non_exhaustive]
pub struct InProgress<'graph, I: Input, C: Ctrl<I>, In: Iterator<Item = I>> {
    /// Reference to the graph we're riding.
    pub graph: &'graph Graph<I, C>,
    /// Iterator over input tokens.
    pub input: In,
    /// Internal stack.
    pub stack: Vec<C>,
    /// Internal state.
    pub ctrl: C,
    /// Output type as we go.
    pub output_t: String,
}

impl<I: Input, C: Ctrl<I>, In: Iterator<Item = I>> fmt::Debug for InProgress<'_, I, C, In> {
    #[inline]
    #[allow(unsafe_code)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "In progress: {} @ {:?} -> {}",
            self.stack.to_src(),
            self.ctrl.view().collect::<Vec<_>>(),
            self.output_t,
        )
    }
}

/// Input intentionally rejected by a parser without anything going wrong internally.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum InputError {
    /// Ended in a non-accepting state.
    NotAccepting,
    /// Ended input with items in the stack.
    Unclosed,
    /// Tried to close a region that hadn't been opened.
    Unopened,
    /// Tried to take a transition that did not exist.
    Absurd,
}

/// Either the parser intentionally rejected the input or the parser was broken.
#[allow(clippy::exhaustive_enums)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError<I: Input, C: Ctrl<I>> {
    /// Input intentionally rejected by a parser without anything going wrong internally.
    BadInput(InputError),
    /// Parser was broken.
    BadParser(IllFormed<I, C>),
}

impl<I: Input, C: Ctrl<I>, In: Iterator<Item = I>> Iterator for InProgress<'_, I, C, In> {
    type Item = Result<I, ParseError<I, C>>;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let maybe_token = self.input.next();
        let (c, o) = match step(
            self.graph,
            &self.ctrl,
            maybe_token.clone(),
            &mut self.stack,
            &self.output_t,
        ) {
            Ok(ok) => ok,
            Err(e) => return Some(Err(e)),
        };
        self.output_t = o;
        self.ctrl = c?;
        maybe_token.map(Ok) // <-- Propagate the iterator's input
    }
}

/// Act on the automaton graph in response to one input token.
#[inline]
#[allow(clippy::type_complexity)]
fn step<I: Input, C: Ctrl<I>>(
    graph: &Graph<I, C>,
    ctrl: &C,
    maybe_token: Option<I>,
    stack: &mut Vec<C>,
    output_t: &str,
) -> Result<(Option<C>, String), ParseError<I, C>> {
    ctrl.view().try_fold((), |(), i| {
        if graph.states.get(i).is_none() {
            Err(ParseError::BadParser(IllFormed::OutOfBounds(i)))
        } else {
            Ok(())
        }
    })?;
    let mut states = ctrl.view().map(|i| get!(graph.states, i));
    let Some(token) = maybe_token else {
        return if stack.is_empty() {
            if states.any(|s| s.non_accepting.is_empty()) {
                Ok((None, output_t.to_owned()))
            } else {
                Err(ParseError::BadInput(InputError::NotAccepting))
            }
        } else {
            Err(ParseError::BadInput(InputError::Unclosed))
        };
    };

    // Merge into a huge aggregate transition and act on that instead of individual transitions
    match try_merge(states.filter_map(|s| match s.transitions.get(&token) {
        Err(e) => Some(Err(e)),
        Ok(opt) => opt.map(|t| Ok(t.clone())),
    })) {
        None => Err(ParseError::BadInput(InputError::Absurd)),
        Some(Err(e)) => Err(ParseError::BadParser(e)),
        Some(Ok(mega_transition)) => mega_transition.invoke(output_t, stack)?.map_or(
            Err(ParseError::BadInput(InputError::Unopened)),
            |(c, out)| Ok((Some(c), out)),
        ),
    }
}
