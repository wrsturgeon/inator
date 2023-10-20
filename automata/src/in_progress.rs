/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Execute an automaton on an input sequence.

use crate::{try_merge, Ctrl, Graph, IllFormed, Input, Output, Stack, Transition};
use core::{fmt, mem};

/// Execute an automaton on an input sequence.
#[non_exhaustive]
pub struct InProgress<
    'graph,
    I: Input,
    S: Stack,
    O: Output,
    C: Ctrl<I, S, O>,
    In: Iterator<Item = I>,
> {
    /// Reference to the graph we're riding.
    pub graph: &'graph Graph<I, S, O, C>,
    /// Iterator over input tokens.
    pub input: In,
    /// Internal stack.
    pub stack: Vec<S>,
    /// Internal state.
    pub ctrl: Result<C, bool>,
    /// Output accumulator.
    pub output: mem::MaybeUninit<O>,
}

impl<
        I: Input,
        S: fmt::Debug + Stack,
        O: fmt::Debug + Output,
        C: Ctrl<I, S, O>,
        In: Iterator<Item = I>,
    > fmt::Debug for InProgress<'_, I, S, O, C, In>
{
    #[inline]
    #[allow(unsafe_code)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "In progress: {:?} @ {:?} -> {:?}",
            self.stack,
            self.ctrl.as_ref().map(|c| c.view().collect::<Vec<_>>()),
            // SAFETY: Never uninitialized except inside one function (and initialized before it exits).
            unsafe { self.output.assume_init_ref() },
        )
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>, In: Iterator<Item = I>> Iterator
    for InProgress<'_, I, S, O, C, In>
{
    type Item = Result<I, IllFormed<I, S, O, C>>;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let maybe_token = self.input.next();
        if let Ok(ref ctrl) = self.ctrl {
            let (c, o) = match step(
                self.graph,
                ctrl,
                maybe_token.as_ref(),
                &mut self.stack,
                #[allow(unsafe_code)]
                // SAFETY: All good: nowhere else uninitialized and initialized later in this function.
                unsafe {
                    mem::replace(&mut self.output, mem::MaybeUninit::uninit()).assume_init()
                },
            ) {
                Ok(ok) => ok,
                Err(e) => return Some(Err(e)),
            };
            self.ctrl = c;
            let _ = self.output.write(o);
        }
        maybe_token.map(Ok) // <-- Propagate the iterator's input
    }
}

/// Act on the automaton graph in response to one input token.
#[inline]
#[allow(clippy::type_complexity)]
fn step<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>>(
    graph: &Graph<I, S, O, C>,
    ctrl: &C,
    maybe_token: Option<&I>,
    stack: &mut Vec<S>,
    output: O,
) -> Result<(Result<C, bool>, O), IllFormed<I, S, O, C>> {
    ctrl.view().try_fold((), |(), i| {
        if graph.states.get(i).is_none() {
            Err(IllFormed::OutOfBounds(i))
        } else {
            Ok(())
        }
    })?;
    let mut states = ctrl.view().map(|i| get!(graph.states, i));
    let Some(token) = maybe_token else {
        return Ok((Err(stack.is_empty() && states.any(|s| s.accepting)), output));
    };
    let maybe_stack_top = stack.last();
    let transitions = states.filter_map(|s| match s.transitions.get(maybe_stack_top, token) {
        Err(e) => Some(Err(e)),
        Ok(opt) => opt.map(Ok),
    });
    let mega_transition: Transition<I, S, O, C> = match try_merge(transitions) {
        None => return Ok((Err(false), output)),
        Some(r) => r?,
    };
    Ok(mega_transition.invoke(token, stack, output))
}
