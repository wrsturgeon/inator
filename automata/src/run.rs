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
    /// # Errors
    /// If the automaton is not well-formed (with a witness to why).
    #[allow(clippy::type_complexity)]
    fn run<C: Ctrl<Self::Item>>(
        self,
        graph: &Graph<Self::Item, C>,
    ) -> InProgress<'_, Self::Item, C, Self::IntoIter>;
}

impl<In: IntoIterator> Run for In
where
    In::Item: Input,
{
    #[inline]
    fn run<C: Ctrl<Self::Item>>(
        self,
        graph: &Graph<Self::Item, C>,
    ) -> InProgress<'_, Self::Item, C, Self::IntoIter> {
        InProgress {
            graph,
            input: self.into_iter(),
            stack: vec![],
            ctrl: graph.initial.clone(),
            output_t: "()".to_owned(),
        }
    }
}
