/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Execute an automaton on an input sequence.

use crate::{find_tag, try_merge, Ctrl, Graph, IllFormed, Input, Output, Stack};
use core::{fmt, iter, mem};

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
    pub ctrl: C,
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
            self.ctrl.view().collect::<Vec<_>>(),
            // SAFETY: Never uninitialized except inside one function (and initialized before it exits).
            unsafe { self.output.assume_init_ref() },
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
pub enum ParseError<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> {
    /// Input intentionally rejected by a parser without anything going wrong internally.
    BadInput(InputError),
    /// Parser was broken.
    BadParser(IllFormed<I, S, O, C>),
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>, In: Iterator<Item = I>> Iterator
    for InProgress<'_, I, S, O, C, In>
{
    type Item = Result<I, ParseError<I, S, O, C>>;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let maybe_token = self.input.next();
        let (c, o) = match step(
            self.graph,
            &self.ctrl,
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
        let _ = self.output.write(o);
        self.ctrl = c?;
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
) -> Result<(Option<C>, O), ParseError<I, S, O, C>> {
    ctrl.view().try_fold((), |(), r| match r {
        Ok(i) => {
            if graph.states.get(i).is_none() {
                Err(ParseError::BadParser(IllFormed::OutOfBounds(i)))
            } else {
                Ok(())
            }
        }
        Err(s) => find_tag(&graph.states, s)
            .map(|_| {})
            .map_err(ParseError::BadParser),
    })?;
    let mut states = ctrl.view().flat_map(|r| match r {
        Ok(i) => iter::once(get!(graph.states, i)).collect(),
        Err(s) => find_tag(&graph.states, s).unwrap_or_else(|_| never!()),
    });
    let Some(token) = maybe_token else {
        return if stack.is_empty() {
            if states.any(|s| s.accepting) {
                Ok((None, output))
            } else {
                Err(ParseError::BadInput(InputError::NotAccepting))
            }
        } else {
            Err(ParseError::BadInput(InputError::Unclosed))
        };
    };
    let maybe_stack_top = stack.last();
    let transitions = states.filter_map(|s| match s.transitions.get(maybe_stack_top, token) {
        Err(e) => Some(Err(e)),
        Ok(opt) => opt.map(Ok),
    });
    try_merge(transitions).map_or(Err(ParseError::BadInput(InputError::Absurd)), |r| {
        r.map_or_else(
            |e| Err(ParseError::BadParser(e)),
            |mega_transition| {
                mega_transition.invoke(token, stack, output).map_or(
                    Err(ParseError::BadInput(InputError::Unopened)),
                    |(c, out)| Ok((Some(c), out)),
                )
            },
        )
    })
}
