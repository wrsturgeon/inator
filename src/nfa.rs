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
    pub(crate) non_epsilon: BTreeMap<I, BTreeSet<usize>>,
    /// Whether an input that ends in this state ought to be accepted.
    pub(crate) accepting: bool,
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

impl<I: Clone + Ord> Graph<I> {
    /// NFA accepting this exact character and only this exact character, only once.
    #[must_use]
    #[inline]
    pub fn unit(singleton: I) -> Self {
        Self {
            states: vec![
                State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: core::iter::once((singleton, core::iter::once(1).collect()))
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

    /// Check if there are any states (empty would be illegal, but hey, why crash your program).
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.states.is_empty() || self.initial.is_empty()
    }

    /// Get the state at a given index.
    #[must_use]
    #[inline(always)]
    pub fn get(&self, i: usize) -> Option<&State<I>> {
        self.states.get(i)
    }

    /// Take every transition that doesn't require input.
    #[inline]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn take_all_epsilon_transitions(&self, mut queue: BTreeSet<usize>) -> BTreeSet<usize> {
        // Take all epsilon transitions immediately
        let mut superposition = BTreeSet::<usize>::new();
        while let Some(state) = queue.pop_first() {
            for next in get!(self.states, state).epsilon_transitions() {
                if !superposition.contains(next) {
                    let _ = queue.insert(*next);
                }
            }
            let _ = superposition.insert(state);
        }
        superposition
    }

    /// Decide whether an input belongs to the regular langage this NFA accepts.
    #[inline]
    #[cfg(test)]
    #[allow(clippy::missing_panics_doc)]
    pub(crate) fn accept<Iter: IntoIterator<Item = I>>(&self, iter: Iter) -> bool {
        if self.is_empty() {
            return false;
        }
        let mut state = self.initial.clone();
        for input in iter {
            state = self
                .take_all_epsilon_transitions(state)
                .into_iter()
                .flat_map(|index| {
                    get!(self.states, index)
                        .transition(&input)
                        .map_or(BTreeSet::new(), Clone::clone)
                })
                .collect();
        }
        self.take_all_epsilon_transitions(state)
            .into_iter()
            .any(|index| get!(self.states, index).accepting)
    }

    /// Number of states.
    #[must_use]
    #[inline(always)]
    pub fn size(&self) -> usize {
        self.states.len()
    }
}

impl<I: Clone + Ord + core::fmt::Display> core::fmt::Display for Graph<I> {
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

impl<I: Clone + Ord + core::fmt::Display> core::fmt::Display for State<I> {
    #[inline]
    #[allow(clippy::use_debug)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(
            f,
            "({}accepting):",
            if self.accepting { "" } else { "NOT " }
        )?;
        writeln!(f, "    epsilon --> {:?}", self.epsilon)?;
        for (input, transitions) in &self.non_epsilon {
            writeln!(f, "    {input} --> {transitions:?}")?;
        }
        Ok(())
    }
}

impl<I: Clone + Ord> State<I> {
    /// Valid inputs mapped to the set of states to which this state can transition on that input.
    #[inline(always)]
    pub const fn non_epsilon_transitions(&self) -> &BTreeMap<I, BTreeSet<usize>> {
        &self.non_epsilon
    }

    /// Set of states to which this state can immediately transition without input.
    #[inline(always)]
    pub const fn epsilon_transitions(&self) -> &BTreeSet<usize> {
        &self.epsilon
    }

    /// Set of states to which this state can transition on a given input.
    #[inline]
    pub fn transition(&self, input: &I) -> Option<&BTreeSet<usize>> {
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
fn cut_nonsense<I: Clone + Ord>(v: &mut Vec<State<I>>) {
    let size = v.len();
    for state in v {
        state.epsilon.retain(|i| i < &size);
        for destination in state.non_epsilon.values_mut() {
            destination.retain(|index| index < &size);
        }
    }
}
