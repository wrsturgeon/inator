/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Nondeterministic finite automata with epsilon transitions.

use std::collections::{btree_map::Entry, BTreeMap};

/// Nondeterministic finite automata with epsilon transitions.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Graph<I: Clone + Ord> {
    /// Every state in this graph.
    pub(crate) states: Vec<State<I>>,
    /// Initial set of states.
    pub(crate) initial: BTreeMap<usize, bool>,
}

/// Transitions from one state to arbitrarily many others, possibly without even consuming input.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct State<I: Clone + Ord> {
    /// Transitions that doesn't require consuming input.
    pub(crate) epsilon: BTreeMap<usize, bool>,
    /// Transitions that require consuming and matching input.
    pub(crate) non_epsilon: BTreeMap<I, BTreeMap<usize, Vec<I>>>,
    /// Whether an input that ends in this state ought to be accepted.
    pub(crate) accepting: bool,
}

/// Test if there is a way to split the input such that
/// automaton #1 accepts the left part and #2 accepts the right.
#[inline]
#[cfg(test)]
pub(crate) fn chain<I: Clone + Ord + core::fmt::Debug>(
    a1: &Graph<I>,
    a2: &Graph<I>,
    input: &[I],
) -> Option<Vec<I>> {
    let mut s1 = a1.step();
    let mut i = input.iter();
    if let Some(vl) = s1.currently_accepting() {
        let vr = a2.format(i.clone())?;
        let mut v = vl.clone();
        v.extend(vr);
        return Some(v);
    }
    while let Some(token) = i.next() {
        s1.step(token);
        if let Some(vl) = s1.currently_accepting() {
            let vr = a2.format(i.clone())?;
            let mut v = vl.clone();
            v.extend(vr);
            return Some(v);
        }
    }
    None
}

impl<I: Clone + Ord> IntoIterator for Graph<I> {
    type Item = State<I>;
    type IntoIter = std::vec::IntoIter<State<I>>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.states.into_iter()
    }
}

impl<'a, I: Clone + Ord> IntoIterator for &'a Graph<I> {
    type Item = &'a State<I>;
    type IntoIter = core::slice::Iter<'a, State<I>>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.states.iter()
    }
}

impl<I: Clone + Ord> Default for Graph<I> {
    #[inline(always)]
    fn default() -> Self {
        Self::void()
    }
}

impl<I: Clone + Ord> Graph<I> {
    /// NFA with zero states.
    #[inline]
    #[must_use]
    pub fn void() -> Self {
        Self {
            states: vec![],
            initial: BTreeMap::new(),
        }
    }

    /// NFA accepting only the empty string.
    #[inline]
    #[must_use]
    pub fn empty() -> Self {
        Self {
            states: vec![State {
                epsilon: BTreeMap::new(),
                non_epsilon: BTreeMap::new(),
                accepting: true,
            }],
            initial: core::iter::once((0, true)).collect(),
        }
    }

    /// NFA accepting this exact token and only this exact token, only once.
    #[must_use]
    #[inline]
    pub fn unit(singleton: I, repl: Vec<I>) -> Self {
        Self {
            states: vec![
                State {
                    epsilon: BTreeMap::new(),
                    non_epsilon: core::iter::once((
                        singleton,
                        core::iter::once((1, repl)).collect(),
                    ))
                    .collect(),
                    accepting: false,
                },
                State {
                    epsilon: BTreeMap::new(),
                    non_epsilon: BTreeMap::new(),
                    accepting: true,
                },
            ],
            initial: core::iter::once((0, true)).collect(),
        }
    }

    /// Take every transition that doesn't require input.
    #[inline]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn take_all_epsilon_transitions(
        &self,
        mut queue: BTreeMap<usize, (bool, Vec<I>)>,
    ) -> BTreeMap<usize, (bool, Vec<I>)> {
        // Take all epsilon transitions immediately
        let mut superposition = BTreeMap::new();
        while let Some((next, (proper, repl))) = queue.pop_first() {
            if let Entry::Vacant(entry) = superposition.entry(next) {
                for (&eps_next, &eps_proper) in &get!(self.states, next).epsilon {
                    queue.insert(eps_next, (proper && eps_proper, repl.clone()));
                }
                entry.insert((proper, repl));
            }
        }
        superposition
    }

    /// Step through each input token one at a time.
    #[inline]
    #[must_use]
    // #[cfg(test)] // <-- TODO: REINSTATE
    pub(crate) fn step(&self) -> Stepper<'_, I>
    where
        I: core::fmt::Debug,
    {
        Stepper::new(self)
    }

    /// Decide whether an input belongs to the regular langage this NFA accepts.
    #[inline]
    #[must_use]
    // #[cfg(test)] // <-- TODO: REINSTATE
    #[allow(clippy::missing_panics_doc)]
    // TODO: MAKE `pub(crate)` AGAIN
    pub fn format<Iter: IntoIterator>(&self, iter: Iter) -> Option<Vec<I>>
    where
        Iter::Item: core::borrow::Borrow<I>,
        I: core::fmt::Debug,
    {
        let mut stepper = self.step();
        stepper.extend(iter);
        stepper.take()
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
    pub fn fuzz(&self) -> Result<super::Fuzzer<I>, super::NeverAccepts> {
        super::Fuzzer::try_from_reversed(self.reverse().compile())
    }

    /// Check if there exists a string this DFA will accept.
    #[inline]
    #[must_use]
    pub fn would_ever_accept(&self) -> bool {
        self.states.iter().any(|state| state.accepting) && !self.initial.is_empty()
    }

    /// Match AT LEAST once, then as many times as we want.
    /// If you want to match potentially zero times, use `.star()`.
    /// Note that if ANY number of times leads to an accepting state, we take it!
    #[inline]
    #[must_use]
    pub fn or_more(mut self) -> Self {
        for state in &mut self.states {
            if state.accepting {
                state
                    .epsilon
                    .extend(self.initial.iter().map(|(&i, _)| (i, false)));
            }
        }
        self
    }

    /// Match at most one time and don't write to formatted output.
    #[inline]
    #[must_use]
    pub fn ignore(mut self) -> Self {
        // self.states.push(State {
        //     epsilon: core::mem::replace(
        //         &mut self.initial,
        //         core::iter::once(self.states.len()).collect(),
        //     )
        //     .into_iter()
        //     .map(|i| (i, false))
        //     .collect(),
        //     non_epsilon: BTreeMap::new(),
        //     accepting: true,
        // });
        // self
        for proper in self.initial.values_mut() {
            *proper = false
        }
        self | Self::empty()
    }

    /// Match at most one time and write to formatted output (even if it wasn't there).
    #[inline]
    #[must_use]
    pub fn supply(mut self) -> Self {
        // self.states.push(State {
        //     epsilon: core::mem::replace(
        //         &mut self.initial,
        //         core::iter::once(self.states.len()).collect(),
        //     )
        //     .into_iter()
        //     .map(|i| (i, true))
        //     .collect(),
        //     non_epsilon: BTreeMap::new(),
        //     accepting: true,
        // });
        // self
        let mut empty = Self::empty();
        for proper in empty.initial.values_mut() {
            *proper = false;
        }
        self | empty
    }

    /// Match zero or more times (a.k.a. Kleene star) and never write to formatted output.
    #[inline]
    #[must_use]
    pub fn star(self) -> Self {
        self.or_more().ignore()
    }

    /// Find the minimal input that reaches this state.
    /// Like Dijkstra's, but optimized to leverage that each edge is 0 (if epsilon) or 1 (otherwise).
    #[inline]
    #[must_use]
    #[allow(clippy::panic_in_result_fn, clippy::unwrap_in_result)]
    pub(crate) fn dijkstra<Init: IntoIterator<Item = usize>>(
        &self,
        initial: Init,
        endpoint: usize,
    ) -> Option<Vec<I>> {
        use core::cmp::Reverse;
        use std::collections::BinaryHeap;

        let mut cache = BTreeMap::<usize, Vec<I>>::new();
        let mut queue = BinaryHeap::new();

        for init in initial {
            drop(cache.insert(init, vec![]));
            queue.push(Reverse(CmpFirst(0_usize, init)));
        }

        while let Some(Reverse(CmpFirst(distance, index))) = queue.pop() {
            let mut cached = unwrap!(cache.get(&index)).clone(); // TODO: look into `Cow`
            let state = get!(self.states, index);
            for (&next, &formatted) in &state.epsilon {
                if next == endpoint {
                    return Some(cached);
                }
                if let Entry::Vacant(entry) = cache.entry(next) {
                    let _ = entry.insert(cached.clone());
                    queue.push(Reverse(CmpFirst(distance, next)));
                }
            }
            for (token, repl) in &state.non_epsilon {
                for (&next, formatted) in repl {
                    if next == endpoint {
                        cached.push(token.clone());
                        return Some(cached);
                    }
                    if let Entry::Vacant(entry) = cache.entry(next) {
                        entry.insert(cached.clone()).push(token.clone());
                        queue.push(Reverse(CmpFirst(distance.saturating_add(1), next)));
                    }
                }
            }
        }

        None
    }

    /// Find the minimal input that reaches this state.
    #[inline]
    #[must_use]
    // #[cfg(test)] // <-- TODO: REINSTATE
    #[allow(clippy::panic_in_result_fn, clippy::unwrap_in_result)]
    pub(crate) fn backtrack(&self, endpoint: usize) -> Option<Vec<I>> {
        self.dijkstra(self.initial.iter().map(|(&k, _)| k), endpoint)
    }
}

/// Only the first element matters for equality and comparison.
#[derive(Clone, Copy, Debug, Default)]
struct CmpFirst<A: Ord, B>(pub(crate) A, pub(crate) B);

impl<A: Ord, B> PartialEq for CmpFirst<A, B> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<A: Ord, B> Eq for CmpFirst<A, B> {}

impl<A: Ord, B> PartialOrd for CmpFirst<A, B> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<A: Ord, B> Ord for CmpFirst<A, B> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<I: Clone + Ord + core::fmt::Debug + core::fmt::Display> core::fmt::Display for Graph<I> {
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

impl<I: Clone + Ord + core::fmt::Debug + core::fmt::Display> core::fmt::Display for State<I> {
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
        for (input, transitions) in &self.non_epsilon {
            writeln!(f, "    {input} --> {transitions:?}")?;
        }
        Ok(())
    }
}

#[cfg(feature = "quickcheck")]
#[allow(clippy::arithmetic_side_effects)]
impl<I: Ord + quickcheck::Arbitrary> quickcheck::Arbitrary for Graph<I> {
    #[inline]
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let mut states = quickcheck::Arbitrary::arbitrary(g);
        cut_nonsense(&mut states);
        let mut initial = BTreeMap::arbitrary(g);
        initial = initial
            .into_iter()
            .map(|(k, v)| (k % states.len(), v))
            .collect();
        Self { states, initial }
    }

    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new((self.states.clone(), self.initial.clone()).shrink().map(
            |(mut states, mut initial)| {
                cut_nonsense(&mut states);
                initial = initial.into_iter().map(|i| i % states.len()).collect();
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
            non_epsilon: quickcheck::Arbitrary::arbitrary(g),
            accepting: quickcheck::Arbitrary::arbitrary(g),
        }
    }

    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (
                self.epsilon.clone(),
                self.non_epsilon.clone(),
                self.accepting,
            )
                .shrink()
                .map(|(epsilon, non_epsilon, accepting)| Self {
                    epsilon,
                    non_epsilon,
                    accepting,
                }),
        )
    }
}

/// Remove impossible transitions from automatically generated automata.
#[cfg(feature = "quickcheck")]
#[allow(clippy::arithmetic_side_effects)]
fn cut_nonsense<I: Clone + Ord>(v: &mut Vec<State<I>>) {
    let size = v.len();
    for state in v {
        state.epsilon = state.epsilon.iter().map(|i| i % size).collect();
        for &mut Replace { ref mut set, .. } in state.non_epsilon.values_mut() {
            *set = set.iter().map(|i| i % size).collect();
        }
    }
}

/// Step through an automaton one token at a time.
// #[cfg(test)] // <-- TODO: REINSTATE
pub(crate) struct Stepper<'graph, I: Clone + Ord> {
    /// The graph we're riding.
    graph: &'graph Graph<I>,
    /// Current states after the input we've received so far.
    threads: BTreeMap<usize, (bool, Vec<I>)>,
}

// #[cfg(test)] // <-- TODO: REINSTATE
#[allow(dead_code)] // <-- TODO: REMOVE
impl<'graph, I: Clone + Ord + core::fmt::Debug> Stepper<'graph, I> {
    /// Start from the empty string on a certain automaton.
    #[inline]
    #[must_use]
    fn new(graph: &'graph Graph<I>) -> Self {
        Self {
            graph,
            threads: graph.take_all_epsilon_transitions(
                graph
                    .initial
                    .iter()
                    .map(|(&index, &proper)| (index, (|| -> Vec<_> { todo!() })()))
                    .collect(),
            ),
        }
    }

    /// Append an input token.
    #[inline]
    fn step(&mut self, token: &I) {
        let mut threads = BTreeMap::new();
        for (&index, formatted) in &self.threads {
            if let Some(edges) = get!(self.graph.states, index).non_epsilon.get(token) {
                for (&i, &extend) in edges {
                    let mut append = formatted.clone();
                    append.extend(append.iter().cloned());
                    threads.insert(i, append);
                }
            }
        }
        self.threads = self.graph.take_all_epsilon_transitions(threads);
    }

    /// Check if the automaton accepts the input we've received so far.
    #[inline]
    fn currently_accepting(&self) -> Option<&Vec<I>> {
        let mut winner_winner = None;
        for (&index, formatted) in &self.threads {
            if get!(self.graph.states, index).accepting {
                #[allow(clippy::option_if_let_else)] // Borrowing issues
                match winner_winner {
                    Some(v) => {
                        assert!(
                            v == formatted,
                            "Two irreconcilable outputs on the same valid input: \
                            {:?} produces both {v:?} and {formatted:?}",
                            self.graph.backtrack(index),
                        );
                    }
                    None => winner_winner = Some(formatted),
                }
            }
        }
        winner_winner
    }

    /// Check if the automaton accepts the input we've received so far.
    #[inline]
    fn take(self) -> Option<Vec<I>> {
        let mut winner_winner = None;
        for (index, formatted) in self.threads {
            if get!(self.graph.states, index).accepting {
                match winner_winner {
                    Some(ref v) => {
                        assert!(
                            v == &formatted,
                            "Two irreconcilable outputs on the same valid input: \
                            {:?} produces both {v:?} and {formatted:?}",
                            self.graph.backtrack(index),
                        );
                    }
                    None => winner_winner = Some(formatted),
                }
            }
        }
        winner_winner
    }
}

// #[cfg(test)] // <-- TODO: REINSTATE
impl<I: Clone + Ord + core::fmt::Debug, B: core::borrow::Borrow<I>> Extend<B> for Stepper<'_, I> {
    #[inline]
    fn extend<T: IntoIterator<Item = B>>(&mut self, iter: T) {
        for input in iter {
            self.step(input.borrow());
        }
    }
}
