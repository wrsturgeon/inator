/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Execute an automaton on an input sequence.

use crate::{Ctrl, Graph, InProgress, Input};

/// Execute an automaton on an input sequence.
pub trait Run: IntoIterator + Sized
where
    Self::Item: Input,
{
    /// Execute an automaton on an input sequence.
    #[must_use]
    fn run<S, C: Ctrl>(
        self,
        graph: &Graph<Self::Item, S, C>,
    ) -> InProgress<'_, Self::IntoIter, S, C>;
}

impl<In: IntoIterator> Run for In
where
    In::Item: Input,
{
    #[inline]
    #[must_use]
    fn run<S, C: Ctrl>(
        self,
        graph: &Graph<Self::Item, S, C>,
    ) -> InProgress<'_, Self::IntoIter, S, C> {
        InProgress {
            graph,
            input: self.into_iter(),
            stack: vec![],
            ctrl: Ok(graph.initial.clone()),
        }
    }
}
