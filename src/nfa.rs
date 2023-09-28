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
pub struct Graph<'post, I: Clone + Ord> {
    /// Every state in this graph.
    pub(crate) states: Vec<Postpone<'post, I>>,
    /// Initial set of states.
    pub(crate) initial: BTreeSet<usize>,
}

/// Transitions from one state to arbitrarily many others, possibly without even consuming input.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct State<I: Clone + Ord> {
    /// Transitions that doesn't require consuming input.
    pub(crate) epsilon: BTreeSet<usize>,
    /// Transitions that require consuming and matching input.
    pub(crate) non_epsilon: BTreeMap<I, (BTreeSet<usize>, Option<&'static str>)>,
    /// Whether an input that ends in this state ought to be accepted.
    pub(crate) accepting: bool,
}

/// Guard against infinite recursion.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Postpone<'post, I: Clone + Ord> {
    /// Immediate value.
    Now(State<I>),
    /// Construct a value later and tell us by reference.
    Later(Option<&'post State<I>>),
}

/// Test if there is a way to split the input such that
/// automaton #1 accepts the left part and #2 accepts the right.
#[inline]
#[cfg(test)]
pub(crate) fn chain<'post, I: Clone + Ord>(
    a1: &Graph<'post, I>,
    a2: &Graph<'post, I>,
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

impl<'post, I: Clone + Ord> IntoIterator for Graph<'post, I> {
    type Item = Postpone<'post, I>;
    type IntoIter = std::vec::IntoIter<Postpone<'post, I>>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.states.into_iter()
    }
}

impl<'a, 'post, I: Clone + Ord> IntoIterator for &'a Graph<'post, I> {
    type Item = &'a Postpone<'post, I>;
    type IntoIter = core::slice::Iter<'a, Postpone<'post, I>>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.states.iter()
    }
}

impl<I: Clone + Ord> Default for Graph<'_, I> {
    #[inline(always)]
    fn default() -> Self {
        Self::void()
    }
}

impl<I: Clone + Ord> Graph<'_, I> {
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
            states: vec![Postpone::Now(State {
                epsilon: BTreeSet::new(),
                non_epsilon: BTreeMap::new(),
                accepting: true,
            })],
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
            get!(self.states, state).map_ref(|now| {
                for next in &now.epsilon {
                    if !superposition.contains(next) {
                        queue.push(*next);
                    }
                }
                let _ = superposition.insert(state);
            });
        }
        superposition
    }

    /// Step through each input token one at a time.
    #[inline]
    #[must_use]
    pub fn step(&self) -> Stepper<'_, '_, I> {
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
    pub fn fuzz(&self) -> Result<crate::Fuzzer<I>, crate::NeverAccepts> {
        crate::Fuzzer::try_from_reversed(self.reverse().compile())
    }

    /// Check if there exists a string this DFA will accept.
    #[inline]
    #[must_use]
    pub fn would_ever_accept(&self) -> bool {
        self.states.iter().any(|state| match *state {
            Postpone::Now(ref now) => now.accepting,
            Postpone::Later(_) => false,
        }) && !self.initial.is_empty()
    }

    /// Match at least one time, then as many times as we want.
    /// Note that if ANY number of times leads to an accepting state, we take it!
    #[inline]
    #[must_use]
    pub fn repeat(mut self) -> Self {
        for state in &mut self.states {
            state.map_mut(|now| {
                if now.accepting {
                    now.epsilon.extend(self.initial.iter());
                }
            });
        }
        self
    }

    /// Match at most one time (i.e. ignore if not present).
    #[inline]
    #[must_use]
    pub fn optional(self) -> Self {
        Self::empty() | self
    }

    /// Match zero or more times (a.k.a. Kleene star).
    #[inline]
    #[must_use]
    pub fn star(self) -> Self {
        self.repeat().optional()
    }
}

impl<I: Clone + Ord + Expression> core::fmt::Display for Graph<'_, I> {
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

impl<I: Clone + Ord + Expression> core::fmt::Display for State<I> {
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

impl<I: Clone + Ord + Expression> core::fmt::Display for Postpone<'_, I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::Now(ref now) => core::fmt::Display::fmt(&now, f),
            Self::Later(_) => write!(f, "[POSTPONED]"),
        }
    }
}

impl<I: Clone + Ord> State<I> {
    /// Set of states to which this state can transition on a given input.
    #[inline]
    pub fn transition(&self, input: &I) -> Option<&(BTreeSet<usize>, Option<&'static str>)> {
        self.non_epsilon.get(input)
    }
}

impl<I: Clone + Ord> Postpone<'_, I> {
    /// Apply this function iff this variant is `Self::Now`.
    #[inline]
    pub fn map<F: FnOnce(State<I>)>(self, f: F) {
        if let Self::Now(now) = self {
            f(now);
        }
    }

    /// Apply this function iff this variant is `Self::Now`.
    #[inline]
    pub fn map_ref<F: FnOnce(&State<I>)>(&self, f: F) {
        if let &Self::Now(ref now) = self {
            f(now);
        }
    }

    /// Apply this function iff this variant is `Self::Now`.
    #[inline]
    pub fn map_mut<F: FnOnce(&mut State<I>)>(&mut self, f: F) {
        if let &mut Self::Now(ref mut now) = self {
            f(now);
        }
    }

    /// Unwrap iff this variant is `Self::Now` or `Self::Later(Some(..))`.
    /// # Panics
    /// Otherwise.
    #[inline]
    #[allow(clippy::panic)]
    pub fn unwrap(&self) -> &State<I> {
        match *self {
            Self::Now(ref now) => now,
            Self::Later(Some(ptr)) => ptr,
            Self::Later(None) => {
                panic!("Called `unwrap` on a `Postpone` value that had not been initialized.")
            }
        }
    }

    /// Unwrap iff this variant is `Self::Now` or `Self::Later(Some(..))`.
    /// # Panics
    /// Otherwise.
    #[inline]
    #[allow(clippy::panic)]
    pub fn unwrap_mut(&mut self) -> &mut State<I> {
        match *self {
            Self::Now(ref mut now) => now,
            Self::Later(_) => {
                panic!("Called `unwrap_mut` on a `Postpone` value that was not `Now`.")
            }
        }
    }
}

#[cfg(feature = "quickcheck")]
impl<I: Ord + quickcheck::Arbitrary> quickcheck::Arbitrary for Graph<'static, I> {
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

#[cfg(feature = "quickcheck")]
impl<I: Ord + quickcheck::Arbitrary> quickcheck::Arbitrary for Postpone<'static, I> {
    #[inline]
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self::Now(State::arbitrary(g))
    }

    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let &Self::Now(ref state) = self else {
            #[allow(unsafe_code)]
            // SAFETY: Constructed above.
            unsafe {
                core::hint::unreachable_unchecked()
            }
        };
        Box::new(state.shrink().map(Self::Now))
    }
}

/// Remove impossible transitions from automatically generated automata.
#[cfg(feature = "quickcheck")]
fn cut_nonsense<I: Clone + Ord>(v: &mut Vec<Postpone<'static, I>>) {
    let size = v.len();
    for state in v {
        state.unwrap_mut().epsilon.retain(|i| i < &size);
        for &mut (ref mut destination, _) in state.unwrap_mut().non_epsilon.values_mut() {
            destination.retain(|index| index < &size);
        }
    }
}

/// Step through an automaton one token at a time.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Stepper<'graph, 'post, I: Clone + Ord> {
    /// The graph we're riding.
    graph: &'graph Graph<'post, I>,
    /// Current state after the input we've received so far.
    state: BTreeSet<usize>,
}

impl<'graph, 'post, I: Clone + Ord> Stepper<'graph, 'post, I> {
    /// Start from the empty string on a certain automaton.
    #[inline]
    #[must_use]
    fn new(graph: &'graph Graph<'post, I>) -> Self {
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
                .filter_map(|&index| get!(self.graph.states, index).unwrap().transition(token))
                .flat_map(|&(ref map, _)| map.iter().copied())
                .collect(),
        );
    }

    /// Check if the automaton accepts the input we've received so far.
    #[inline]
    fn currently_accepting(&self) -> bool {
        for &index in &self.state {
            if get!(self.graph.states, index).unwrap().accepting {
                return true;
            }
        }
        false
    }
}

impl<I: Clone + Ord, B: core::borrow::Borrow<I>> Extend<B> for Stepper<'_, '_, I> {
    #[inline]
    fn extend<T: IntoIterator<Item = B>>(&mut self, iter: T) {
        for input in iter {
            self.step(input.borrow());
        }
    }
}
