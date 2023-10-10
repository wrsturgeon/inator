/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Nondeterministic finite automata with epsilon transitions.

#![cfg_attr(test, allow(dead_code))] // <-- FIXME

use crate::{call::Call, range::Range, Expression};
use core::{
    fmt::{self, Debug, Display},
    iter::once,
    slice::Iter,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    vec::IntoIter,
};

#[cfg(test)]
use core::borrow::Borrow;

#[cfg(feature = "quickcheck")]
use quickcheck::*;

/// Subset of states (by their index).
type Subset = BTreeSet<usize>;

/// From a single state, all tokens and the transitions each would induce.
type Transitions<I> = BTreeMap<Range<I>, Transition>;

/// A single edge triggered by a token.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct Transition {
    /// Set of destination states.
    pub(crate) dsts: Subset,
    /// Function (or none) to call on this edge.
    pub(crate) call: Call,
}

/// Nondeterministic finite automata with epsilon transitions.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Graph<I: Clone + Expression + Ord> {
    /// Every state in this graph.
    pub(crate) states: Vec<State<I>>,
    /// Initial set of states.
    pub(crate) initial: Subset,
}

/// Transitions from one state to arbitrarily many others, possibly without even consuming input.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct State<I: Clone + Expression + Ord> {
    /// Transitions that doesn't require consuming input.
    pub(crate) epsilon: Subset,
    /// Transitions that require consuming and matching input.
    pub(crate) non_epsilon: Transitions<I>,
    /// Whether an input that ends in this state ought to be accepted.
    pub(crate) accepting: Option<Call>,
}

/// Test if there is a way to split the input such that
/// automaton #1 accepts the left part and #2 accepts the right.
#[inline]
#[cfg(test)]
pub(crate) fn chain<I: Clone + Expression + Ord>(
    a1: &Graph<I>,
    a2: &Graph<I>,
    input: &[I],
) -> bool {
    let mut s1 = a1.step();
    let mut i = input.iter();
    if s1.currently_accepting() && a2.accept(i.clone()) {
        return true;
    }
    while let Some(token) = i.next() {
        s1.step(token);
        if s1.currently_accepting() && a2.accept(i.clone()) {
            return true;
        }
    }
    false
}

impl<I: Clone + Expression + Ord> IntoIterator for Graph<I> {
    type Item = State<I>;
    type IntoIter = IntoIter<State<I>>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.states.into_iter()
    }
}

impl<'a, I: Clone + Expression + Ord> IntoIterator for &'a Graph<I> {
    type Item = &'a State<I>;
    type IntoIter = Iter<'a, State<I>>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.states.iter()
    }
}

impl<I: Clone + Expression + Ord> Default for Graph<I> {
    #[inline(always)]
    fn default() -> Self {
        Self::void()
    }
}

impl<I: Clone + Expression + Ord> Graph<I> {
    /// NFA with zero states.
    #[inline]
    #[must_use]
    pub(crate) fn void() -> Self {
        Self {
            states: vec![],
            initial: Subset::new(),
        }
    }

    /// NFA accepting only the empty string.
    #[inline]
    #[must_use]
    pub(crate) fn empty() -> Self {
        Self {
            states: vec![State {
                epsilon: Subset::new(),
                non_epsilon: BTreeMap::new(),
                accepting: Some(Call::Pass),
            }],
            initial: once(0).collect(),
        }
    }

    /// NFA accepting this exact token and only this exact token, only once.
    #[inline]
    #[must_use]
    #[allow(clippy::arithmetic_side_effects)]
    pub(crate) fn unit(range: Range<I>, call: Call) -> Self {
        Self::empty() >> (range, call, Self::empty())
    }

    /// Take every transition that doesn't require input.
    #[inline]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub(crate) fn take_all_epsilon_transitions(&self, mut queue: Vec<usize>) -> Subset {
        // Take all epsilon transitions immediately
        let mut superposition = Subset::new();
        while let Some(state) = queue.pop() {
            for next in &get!(self.states, state).epsilon {
                if !superposition.contains(next) {
                    queue.push(*next);
                }
            }
            let _ = superposition.insert(state);
        }
        superposition
    }

    /// Step through each input token one at a time.
    #[inline]
    #[must_use]
    #[cfg(test)]
    pub(crate) fn step(&self) -> Stepper<'_, I> {
        Stepper::new(self)
    }

    /// Decide whether an input belongs to the regular langage this NFA accepts.
    #[inline]
    #[must_use]
    #[cfg(test)]
    #[allow(clippy::missing_panics_doc)]
    pub(crate) fn accept<Iter: IntoIterator>(&self, iter: Iter) -> bool
    where
        Iter::Item: Borrow<I>,
    {
        let mut stepper = self.step();
        stepper.extend(iter);
        stepper.currently_accepting()
    }

    /// Number of states.
    #[must_use]
    #[inline(always)]
    pub fn size(&self) -> usize {
        self.states.len()
    }

    /// Randomly generate inputs that are all guaranteed to be accepted.
    /// NOTE: returns an infinite iterator! `for input in automaton.fuzz()?` will loop forever . . .
    /// # Errors
    /// If this automaton never accepts any input.
    #[inline]
    pub(crate) fn fuzz(&self) -> Result<crate::Fuzzer<I>, crate::NeverAccepts>
    where
        I: Debug,
    {
        crate::Fuzzer::try_from_reversed(self.reverse().compile())
    }

    /// Match at least one time, then as many times as we want.
    /// Note that if ANY number of times leads to an accepting state, we take it!
    #[inline]
    #[must_use]
    pub fn repeat(&self) -> Self {
        let mut s = self.clone();
        for state in &mut s.states {
            if state.accepting.is_some() {
                state.epsilon.extend(s.initial.iter());
            }
        }
        s
    }

    /// Match at most one time (i.e. ignore if not present).
    #[inline]
    #[must_use]
    pub fn optional(&self) -> Self {
        Self::empty() | self.clone()
    }

    /// Match zero or more times (a.k.a. Kleene star).
    #[inline]
    #[must_use]
    pub fn star(&self) -> Self {
        self.repeat().optional()
    }

    /// Remove all calls (set them to `None`).
    #[inline]
    #[must_use]
    #[cfg(test)]
    pub(crate) fn remove_calls(self) -> Self {
        Self {
            states: self.states.into_iter().map(State::remove_calls).collect(),
            ..self
        }
    }
}

impl<I: Clone + Expression + Ord> State<I> {
    /// Set of states to which this state can transition on a given input.
    #[inline]
    #[cfg(test)]
    pub(crate) fn find_transition(&self, input: &I) -> Option<&Transition> {
        // self.non_epsilon.get(input)
        let mut found = None;
        for (range, transition) in &self.non_epsilon {
            if range.contains(input) {
                assert_eq!(
                    found, None,
                    "Duplicate transition (TODO: improve error message here!)",
                );
                found = Some(transition);
            }
        }
        found
    }
}

impl<I: Clone + Expression + Ord> Display for Graph<I> {
    #[inline]
    #[allow(clippy::use_debug)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Initial states: {:?}", self.initial)?;
        for (i, state) in self.states.iter().enumerate() {
            write!(f, "State {i} {state}")?;
        }
        Ok(())
    }
}

impl<I: Clone + Expression + Ord> Display for State<I> {
    #[inline]
    #[allow(clippy::use_debug)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "({}):",
            self.accepting.as_ref().map_or_else(
                || "Not accepting".to_owned(),
                |accept_fn| format!("Accepts with `{accept_fn:?}`")
            )
        )?;
        if !self.epsilon.is_empty() {
            writeln!(f, "    epsilon --> {:?}", self.epsilon)?;
        }
        for (input, &Transition { ref dsts, ref call }) in &self.non_epsilon {
            writeln!(f, "    {input:?} --> {dsts:?} >>= {call:?}")?;
        }
        Ok(())
    }
}

impl<I: Clone + Expression + Ord> State<I> {
    /// Remove all calls (set them to `None`).
    #[inline]
    #[must_use]
    #[cfg(test)]
    pub(crate) fn remove_calls(self) -> Self {
        Self {
            non_epsilon: self
                .non_epsilon
                .into_iter()
                .map(|(token, transition)| (token, transition.remove_calls()))
                .collect(),
            accepting: self.accepting.map(Call::remove_calls),
            ..self
        }
    }
}

impl Transition {
    /// Remove all calls (set them to `None`).
    #[inline]
    #[must_use]
    #[cfg(test)]
    pub(crate) fn remove_calls(self) -> Self {
        Self {
            call: Call::Pass,
            ..self
        }
    }
}

#[cfg(feature = "quickcheck")]
impl<I: Expression + Ord + Arbitrary> Arbitrary for Graph<I> {
    #[inline]
    fn arbitrary(g: &mut Gen) -> Self {
        let mut states = Arbitrary::arbitrary(g);
        cut_nonsense(&mut states);
        let mut initial = Subset::arbitrary(g);
        initial.retain(|i| i < &states.len());
        Self { states, initial }
    }

    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new((self.states.clone(), self.initial.clone()).shrink().map(
            |(mut states, mut initial)| {
                cut_nonsense(&mut states);
                initial.retain(|i| i < &states.len());
                Self { states, initial }
            },
        ))
    }
}

#[cfg(feature = "quickcheck")]
impl<I: Expression + Ord + Arbitrary> Arbitrary for State<I> {
    #[inline]
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            epsilon: Arbitrary::arbitrary(g),
            non_epsilon: BTreeMap::<Range<I>, Subset>::arbitrary(g)
                .into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        Transition {
                            dsts: v,
                            call: Arbitrary::arbitrary(g),
                        },
                    )
                })
                .collect(),
            accepting: Arbitrary::arbitrary(g),
        }
    }

    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (
                self.epsilon.clone(),
                self.non_epsilon
                    .iter()
                    .map(|(token, &Transition { ref dsts, ref call })| {
                        (token.clone(), (dsts.clone(), call.clone()))
                    })
                    .collect::<BTreeMap<_, _>>(),
                self.accepting.clone(),
            )
                .shrink()
                .map(|(epsilon, non_epsilon, accepting)| Self {
                    epsilon,
                    non_epsilon: non_epsilon
                        .into_iter()
                        .map(|(dst, (dsts, call))| (dst, Transition { dsts, call }))
                        .collect(),
                    accepting,
                }),
        )
    }
}

/// Remove impossible transitions from automatically generated automata.
#[cfg(feature = "quickcheck")]
fn cut_nonsense<I: Clone + Expression + Ord>(v: &mut Vec<State<I>>) {
    let size = v.len();
    for state in v {
        state.epsilon.retain(|i| i < &size);
        for &mut Transition { ref mut dsts, .. } in state.non_epsilon.values_mut() {
            dsts.retain(|index| index < &size);
        }
    }
}

/// Step through an automaton one token at a time.
#[cfg(test)]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct Stepper<'graph, I: Clone + Expression + Ord> {
    /// The graph we're riding.
    graph: &'graph Graph<I>,
    /// Current state after the input we've received so far.
    state: Subset,
}

#[cfg(test)]
impl<'graph, I: Clone + Expression + Ord> Stepper<'graph, I> {
    /// Start from the empty string on a certain automaton.
    #[inline]
    #[must_use]
    fn new(graph: &'graph Graph<I>) -> Self {
        Self {
            graph,
            state: graph.take_all_epsilon_transitions(graph.initial.iter().copied().collect()),
        }
    }

    /// Append an input token.
    #[inline]
    fn step(&mut self, token: &I) {
        self.state = self.graph.take_all_epsilon_transitions(
            self.state
                .iter()
                .filter_map(|&index| get!(self.graph.states, index).find_transition(token))
                .flat_map(|&Transition { ref dsts, .. }| dsts.iter().copied())
                .collect(),
        );
    }

    /// Check if the automaton accepts the input we've received so far.
    #[inline]
    fn currently_accepting(&self) -> bool {
        for &index in &self.state {
            if get!(self.graph.states, index).accepting.is_some() {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
impl<I: Clone + Expression + Ord, B: Borrow<I>> Extend<B> for Stepper<'_, I> {
    #[inline]
    fn extend<T: IntoIterator<Item = B>>(&mut self, iter: T) {
        for input in iter {
            self.step(input.borrow());
        }
    }
}
