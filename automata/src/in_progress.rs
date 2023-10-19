/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Execute an automaton on an input sequence.

use crate::{Ctrl, Graph, IllFormed, Input, Transition};
use core::mem;

/// Execute an automaton on an input sequence.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InProgress<'graph, In: Iterator, S, C: Ctrl>
where
    In::Item: Input,
{
    /// Reference to the graph we're riding.
    pub graph: &'graph Graph<In::Item, S, C>,
    /// Iterator over input tokens.
    pub input: In,
    /// Internal stack.
    pub stack: Vec<S>,
    /// Internal state.
    pub ctrl: Result<C, bool>,
}

impl<In: Iterator, S, C: Ctrl> Iterator for InProgress<'_, In, S, C>
where
    In::Item: Input,
{
    type Item = Result<In::Item, IllFormed<In::Item, S, C>>;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let maybe_token = self.input.next();
        if self.ctrl.is_ok() {
            match step(
                self.graph,
                unwrap!(mem::replace(&mut self.ctrl, Err(false))),
                &mut self.stack,
                maybe_token.as_ref(),
            ) {
                Ok(ok) => self.ctrl = ok,
                Err(e) => return Some(Err(e)),
            };
        }
        maybe_token.map(Ok) // <-- Propagate the iterator's input
    }
}

fn step<I: Input, S, C: Ctrl>(
    graph: &Graph<I, S, C>,
    ctrl: C,
    stack: &mut Vec<S>,
    maybe_token: Option<&I>,
) -> Result<Result<C, bool>, IllFormed<I, S, C>> {
    let mut states = ctrl.view().map(|i| get!(graph.states, i));
    let Some(token) = maybe_token else {
        return Ok(Err(stack.is_empty() && states.any(|s| s.accepting)));
    };
    let maybe_stack_top = stack.last();
    let edges = states.filter_map(|s| s.transitions.get((maybe_stack_top, (token, ()))));
    let mega_edge: Transition<I, S, C> = match merge(edges) {
        None => return Ok(Err(false)),
        Some(Err(e)) => return Err(e),
        Some(Ok(ok)) => ok,
    };
    Ok(mega_edge.invoke(stack))
}
