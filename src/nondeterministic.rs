/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Nondeterministic finite automata with epsilon transitions.

use crate::Expression;
use std::collections::{BTreeMap, BTreeSet};

/// Nondeterministic finite automata with epsilon transitions.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Graph<I: Clone + Ord, S: Clone + Ord = ()> {
    /// Every state in this graph.
    pub(crate) states: Vec<State<I, S>>,
    /// Initial set of states.
    pub(crate) initial: BTreeSet<(usize, S)>,
}

/// Transitions from one state to arbitrarily many others, possibly without even consuming input.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct State<I: Clone + Ord, S: Clone + Ord> {
    /// Transitions that doesn't require consuming input.
    pub(crate) epsilon: BTreeSet<usize>,
    /// Transitions that require consuming and matching input.
    pub(crate) non_epsilon: BTreeMap<I, (BTreeSet<usize>, S, Option<&'static str>)>,
    /// Whether an input that ends in this state ought to be accepted.
    pub(crate) accepting: bool,
}

/// Test if there is a way to split the input such that
/// automaton #1 accepts the left part and #2 accepts the right.
#[inline]
#[cfg(test)]
pub(crate) fn chain<I: Clone + Ord, S: Clone + Ord>(
    a1: &Graph<I, S>,
    a2: &Graph<I, S>,
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

impl<I: Clone + Ord, S: Clone + Ord> IntoIterator for Graph<I, S> {
    type Item = State<I, S>;
    type IntoIter = std::vec::IntoIter<State<I, S>>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.states.into_iter()
    }
}

impl<'a, I: Clone + Ord, S: Clone + Ord> IntoIterator for &'a Graph<I, S> {
    type Item = &'a State<I, S>;
    type IntoIter = core::slice::Iter<'a, State<I, S>>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.states.iter()
    }
}

impl<I: Clone + Ord, S: Clone + Ord> Default for Graph<I, S> {
    #[inline(always)]
    fn default() -> Self {
        Self::void()
    }
}

impl<I: Clone + Ord, S: Clone + Ord> Graph<I, S> {
    /// NFA with zero states.
    #[inline]
    #[must_use]
    pub fn void() -> Self {
        Self {
            states: vec![],
            initial: BTreeSet::new(),
        }
    }

    /// NFA accepting only the empty string.
    #[inline]
    #[must_use]
    pub fn empty() -> Self {
        Self {
            states: vec![State {
                epsilon: BTreeSet::new(),
                non_epsilon: BTreeMap::new(),
                accepting: true,
            }],
            initial: core::iter::once(0).collect(),
        }
    }

    /// NFA accepting this exact token and only this exact token, only once.
    #[inline]
    #[must_use]
    #[allow(clippy::arithmetic_side_effects)]
    pub fn unit(singleton: I, fn_name: Option<&'static str>) -> Self {
        Self::empty() >> (singleton, fn_name, Self::empty())
    }

    /// Take every transition that doesn't require input.
    #[inline]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn take_all_epsilon_transitions(&self, mut queue: Vec<usize>) -> BTreeSet<usize> {
        // Take all epsilon transitions immediately
        let mut superposition = BTreeSet::<usize>::new();
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
    pub fn step(&self) -> Stepper<'_, I, S> {
        Stepper::new(self)
    }

    /// Decide whether an input belongs to the regular langage this NFA accepts.
    #[inline]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn accept<Iter: IntoIterator>(&self, iter: Iter) -> bool
    where
        Iter::Item: core::borrow::Borrow<I>,
    {
        let mut stepper = self.step();
        stepper.extend(iter);
        stepper.currently_accepting()
    }

    /// Decide whether an input belongs to the regular langage this NFA accepts.
    #[inline(always)]
    pub fn reject<Iter: IntoIterator<Item = I>>(&self, iter: Iter) -> bool {
        !self.accept(iter)
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
    pub fn fuzz(&self) -> Result<crate::Fuzzer<I, S>, crate::NeverAccepts>
    where
        I: core::fmt::Debug,
    {
        crate::Fuzzer::try_from_reversed(self.reverse().compile())
    }

    /// Check if there exists a string this DFA will accept.
    #[inline]
    #[must_use]
    pub fn would_ever_accept(&self) -> bool {
        self.states.iter().any(|state| state.accepting) && !self.initial.is_empty()
    }

    // /// Find the minimal input that reaches this state.
    // /// Like Dijkstra's but optimized to leverage that each edge is 1 unit long
    // #[inline]
    // #[must_use]
    // #[allow(clippy::panic_in_result_fn, clippy::unwrap_in_result)]
    // pub(crate) fn dijkstra(&self, initial: BTreeSet<usize>, endpoint: BTreeSet<usize>) -> Vec<I> {
    //     use crate::deterministic::CmpFirst;
    //     use core::cmp::Reverse;
    //     use std::collections::{btree_map::Entry, BinaryHeap};

    //     let mut cache = BTreeMap::new();
    //     let mut queue = BinaryHeap::new();

    //     drop(cache.insert(initial.clone(), vec![]));
    //     queue.push(Reverse(CmpFirst(0_usize, initial)));

    //     while let Some(Reverse(CmpFirst(distance, indices))) = queue.pop() {
    //         let cached = unwrap!(cache.get(&indices)).clone(); // TODO: look into `Cow`
    //         if indices == endpoint {
    //             return cached;
    //         }
    //         let subset = self.take_all_epsilon_transitions(indices.into_iter().collect());
    //         if subset == endpoint {
    //             return cached;
    //         }
    //         let states = subset.into_iter().map(|i| get!(self.states, i));
    //         for (token, &(ref pre_eps, _fn_name)) in states.flat_map(|state| &state.non_epsilon) {
    //             let next = self.take_all_epsilon_transitions(pre_eps.iter().copied().collect());
    //             if let Entry::Vacant(entry) = cache.entry(next.clone()) {
    //                 entry.insert(cached.clone()).push(token.clone());
    //                 queue.push(Reverse(CmpFirst(distance.saturating_add(1), next.clone())));
    //             }
    //         }
    //     }

    //     #[allow(clippy::unreachable)]
    //     #[cfg(any(test, debug_assertions))]
    //     {
    //         unreachable!()
    //     }

    //     #[allow(unsafe_code)]
    //     #[cfg(not(any(test, debug_assertions)))]
    //     unsafe {
    //         core::hint::unreachable_unchecked()
    //     }
    // }

    // /// Find the minimal input that reaches this state.
    // #[inline]
    // #[must_use]
    // pub(crate) fn backtrack(&self, endpoint: BTreeSet<usize>) -> Vec<I> {
    //     self.dijkstra(self.initial.clone(), endpoint)
    // }

    /// Match at least one time, then as many times as we want.
    /// Note that if ANY number of times leads to an accepting state, we take it!
    #[inline]
    #[must_use]
    pub fn repeat(&self) -> Self {
        let mut s = self.clone();
        for state in &mut s.states {
            if state.accepting {
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
}

impl<I: Clone + Ord + Expression, S: Clone + Ord> core::fmt::Display for Graph<I, S> {
    #[inline]
    #[allow(clippy::use_debug)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Initial states: {:?}", self.initial)?;
        for (i, state) in self.states.iter().enumerate() {
            write!(f, "State {i} {state}")?;
        }
        Ok(())
    }
}

impl<I: Clone + Ord + Expression, S: Clone + Ord> core::fmt::Display for State<I, S> {
    #[inline]
    #[allow(clippy::use_debug)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(
            f,
            "({}accepting):",
            if self.accepting { "" } else { "NOT " }
        )?;
        if !self.epsilon.is_empty() {
            writeln!(f, "    epsilon --> {:?}", self.epsilon)?;
        }
        for (input, &(ref transitions, fn_name)) in &self.non_epsilon {
            writeln!(f, "    {input:?} --> {transitions:?} >>= {fn_name:?}")?;
        }
        Ok(())
    }
}

impl<I: Clone + Ord, S: Clone + Ord> State<I, S> {
    /// Set of states to which this state can transition on a given input.
    #[inline]
    pub fn transition(&self, input: &I) -> Option<&(BTreeSet<usize>, Option<&'static str>)> {
        self.non_epsilon.get(input)
    }
}

#[cfg(feature = "quickcheck")]
impl<I: Ord + quickcheck::Arbitrary> quickcheck::Arbitrary for Graph<I> {
    #[inline]
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let mut states = quickcheck::Arbitrary::arbitrary(g);
        cut_nonsense(&mut states);
        let mut initial = BTreeSet::arbitrary(g);
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
impl<I: Ord + quickcheck::Arbitrary> quickcheck::Arbitrary for State<I> {
    #[inline]
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self {
            epsilon: quickcheck::Arbitrary::arbitrary(g),
            non_epsilon: BTreeMap::<I, BTreeSet<usize>>::arbitrary(g)
                .into_iter()
                .map(|(k, v)| (k, (v, None)))
                .collect(),
            accepting: quickcheck::Arbitrary::arbitrary(g),
        }
    }

    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (
                self.epsilon.clone(),
                self.non_epsilon
                    .iter()
                    .map(|(token, &(ref map, _))| (token.clone(), map.clone()))
                    .collect::<BTreeMap<_, _>>(),
                self.accepting,
            )
                .shrink()
                .map(|(epsilon, non_epsilon, accepting)| Self {
                    epsilon,
                    non_epsilon: non_epsilon
                        .into_iter()
                        .map(|(dst, set)| (dst, (set, None)))
                        .collect(),
                    accepting,
                }),
        )
    }
}

/// Remove impossible transitions from automatically generated automata.
#[cfg(feature = "quickcheck")]
fn cut_nonsense<I: Clone + Ord>(v: &mut Vec<State<I>>) {
    let size = v.len();
    for state in v {
        state.epsilon.retain(|i| i < &size);
        for &mut (ref mut destination, _) in state.non_epsilon.values_mut() {
            destination.retain(|index| index < &size);
        }
    }
}

/// Step through an automaton one token at a time.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Stepper<'graph, I: Clone + Ord, S: Clone + Ord> {
    /// The graph we're riding.
    graph: &'graph Graph<I, S>,
    /// Current state after the input we've received so far.
    state: BTreeSet<usize>,
}

impl<'graph, I: Clone + Ord, S: Clone + Ord> Stepper<'graph, I, S> {
    /// Start from the empty string on a certain automaton.
    #[inline]
    #[must_use]
    fn new(graph: &'graph Graph<I, S>) -> Self {
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
                .filter_map(|&index| get!(self.graph.states, index).transition(token))
                .flat_map(|&(ref map, _)| map.iter().copied())
                .collect(),
        );
    }

    /// Check if the automaton accepts the input we've received so far.
    #[inline]
    fn currently_accepting(&self) -> bool {
        for &index in &self.state {
            if get!(self.graph.states, index).accepting {
                return true;
            }
        }
        false
    }
}

impl<I: Clone + Ord, S: Clone + Ord, B: core::borrow::Borrow<I>> Extend<B> for Stepper<'_, I, S> {
    #[inline]
    fn extend<T: IntoIterator<Item = B>>(&mut self, iter: T) {
        for input in iter {
            self.step(input.borrow());
        }
    }
}
