/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Nondeterministic finite automata with epsilon transitions.

use std::collections::{BTreeMap, BTreeSet};

/// Nondeterministic finite automata with epsilon transitions.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Graph<I: Clone + Ord> {
    /// Every state in this graph.
    pub(crate) states: Vec<State<I>>,
    /// Initial set of states.
    pub(crate) initial: BTreeSet<usize>,
}

/// Transitions from one state to arbitrarily many others, possibly without even consuming input.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct State<I: Clone + Ord> {
    /// Transitions that doesn't require consuming input.
    pub(crate) epsilon: BTreeSet<usize>,
    /// Transitions that require consuming and matching input.
    pub(crate) non_epsilon: BTreeMap<I, Recommendation<I>>,
    /// Whether an input that ends in this state ought to be accepted.
    pub(crate) accepting: bool,
}

/// Set with a recommended element.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct Recommendation<I: Clone + Ord> {
    /// Set of all elements.
    pub(crate) set: BTreeSet<usize>,
    /// What to append to our running output vector when we take this transition.
    pub(crate) append: Vec<I>,
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
        Self::empty()
    }
}

impl<I: Clone + Ord> Graph<I> {
    /// NFA with zero states.
    #[inline]
    #[must_use]
    pub fn empty() -> Self {
        Self {
            states: vec![],
            initial: BTreeSet::new(),
        }
    }

    /// NFA accepting this exact token and only this exact token, only once.
    #[must_use]
    #[inline]
    pub fn unit(singleton: I, append: Vec<I>) -> Self {
        Self {
            states: vec![
                State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: core::iter::once((
                        singleton,
                        Recommendation {
                            set: core::iter::once(1).collect(),
                            append,
                        },
                    ))
                    .collect(),
                    accepting: false,
                },
                State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: BTreeMap::new(),
                    accepting: true,
                },
            ],
            initial: core::iter::once(0).collect(),
        }
    }

    /// Take every transition that doesn't require input.
    #[inline]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn take_all_epsilon_transitions(
        &self,
        mut queue: Vec<super::dfa::Recommendation<I>>,
    ) -> BTreeSet<super::dfa::Recommendation<I>> {
        // Take all epsilon transitions immediately
        let mut superposition = BTreeSet::new();
        while let Some(rec) = queue.pop() {
            for next in &get!(self.states, rec.next_state).epsilon {
                if !superposition.contains(&rec) {
                    queue.push(super::dfa::Recommendation {
                        next_state: *next,
                        append: rec.append.clone(),
                    });
                }
            }
            let _ = superposition.insert(rec);
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
    pub fn repeat(mut self) -> Self {
        for state in &mut self.states {
            if state.accepting {
                state.epsilon.extend(self.initial.iter());
            }
        }
        self
    }

    /// Match at most one time (i.e. ignore if not present).
    #[inline]
    #[must_use]
    pub fn optional(mut self) -> Self {
        self.states.push(State {
            epsilon: core::mem::replace(
                &mut self.initial,
                core::iter::once(self.states.len()).collect(),
            ),
            non_epsilon: BTreeMap::new(),
            accepting: true,
        });
        self
    }

    /// Match zero or more times (a.k.a. Kleene star).
    #[inline]
    #[must_use]
    pub fn star(self) -> Self {
        self.repeat().optional()
    }

    /// Find the minimal input that reaches this state.
    /// Like Dijkstra's, but each edge is 0 (if epsilon) or 1 (otherwise).
    #[inline]
    #[must_use]
    // #[cfg(test)] // <-- TODO: REINSTATE
    #[allow(clippy::panic_in_result_fn, clippy::unwrap_in_result)]
    pub(crate) fn backtrack(&self, endpoint: usize) -> Option<Vec<I>> {
        use core::cmp::Reverse;
        use std::collections::{btree_map::Entry, BinaryHeap};

        let mut cache = BTreeMap::<usize, Vec<I>>::new();
        let mut queue = BinaryHeap::new();

        for &init in &self.initial {
            drop(cache.insert(init, vec![]));
            queue.push(Reverse(CmpFirst(0_usize, init)));
        }

        while let Some(Reverse(CmpFirst(distance, index))) = queue.pop() {
            let mut cached = unwrap!(cache.get(&index)).clone(); // TODO: look into `Cow`
            let state = get!(self.states, index);
            for &next in &state.epsilon {
                if next == endpoint {
                    return Some(cached);
                }
                if let Entry::Vacant(entry) = cache.entry(next) {
                    let _ = entry.insert(cached.clone());
                    queue.push(Reverse(CmpFirst(distance, next)));
                }
            }
            for (token, &Recommendation { ref set, .. }) in &state.non_epsilon {
                for &next in set {
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

impl<I: Clone + Ord> Recommendation<I> {
    /// Empty.
    #[inline]
    pub(crate) const fn empty() -> Self {
        Self {
            set: BTreeSet::new(),
            append: vec![],
        }
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
            writeln!(f, "    {input} --> {transitions}")?;
        }
        Ok(())
    }
}

impl<I: Clone + Ord> Default for Recommendation<I> {
    #[inline(always)]
    fn default() -> Self {
        Self::empty()
    }
}

impl<I: Clone + Ord + core::fmt::Debug + core::fmt::Display> core::fmt::Display
    for Recommendation<I>
{
    #[inline]
    #[allow(clippy::use_debug)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?} (recommends {:?})", self.set, self.append)
    }
}

#[cfg(feature = "quickcheck")]
#[allow(clippy::arithmetic_side_effects)]
impl<I: Ord + quickcheck::Arbitrary> quickcheck::Arbitrary for Graph<I> {
    #[inline]
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let mut states = quickcheck::Arbitrary::arbitrary(g);
        cut_nonsense(&mut states);
        let mut initial = BTreeSet::arbitrary(g);
        initial = initial.into_iter().map(|i| i % states.len()).collect();
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

#[cfg(feature = "quickcheck")]
impl<I: Ord + quickcheck::Arbitrary> quickcheck::Arbitrary for Recommendation<I> {
    #[inline]
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self {
            set: quickcheck::Arbitrary::arbitrary(g),
            append: quickcheck::Arbitrary::arbitrary(g),
        }
    }

    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.set.clone(), self.append.clone())
                .shrink()
                .map(|(set, append)| Self { set, append }),
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
        for &mut Recommendation { ref mut set, .. } in state.non_epsilon.values_mut() {
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
    state: BTreeSet<super::dfa::Recommendation<I>>,
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
            state: graph.take_all_epsilon_transitions(
                graph
                    .initial
                    .iter()
                    .map(|&i| super::dfa::Recommendation {
                        next_state: i,
                        append: vec![],
                    })
                    .collect(),
            ),
        }
    }

    /// Append an input token.
    #[inline]
    fn step(&mut self, token: &I) {
        let mut v = vec![];
        for &super::dfa::Recommendation {
            next_state: index,
            append: ref formatted,
        } in &self.state
        {
            if let Some(rec) = get!(self.graph.states, index).non_epsilon.get(token) {
                for &i in &rec.set {
                    let mut append = formatted.clone();
                    append.extend(rec.append.iter().cloned());
                    v.push(super::dfa::Recommendation {
                        next_state: i,
                        append,
                    });
                }
            }
        }
        self.state = self.graph.take_all_epsilon_transitions(v);
    }

    /// Check if the automaton accepts the input we've received so far.
    #[inline]
    fn currently_accepting(&self) -> Option<&Vec<I>> {
        let mut winner_winner = None;
        for &super::dfa::Recommendation {
            next_state: index,
            append: ref formatted,
        } in &self.state
        {
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
        for super::dfa::Recommendation {
            next_state: index,
            append: formatted,
        } in self.state
        {
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
