/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Deterministic finite automata.

use std::collections::BTreeMap;

/// Deterministic finite automata.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Graph<I: Clone + Ord> {
    /// Every state in this graph.
    pub(crate) states: Vec<State<I>>,
    /// Initial set of states.
    pub(crate) initial: usize,
}

/// State transitions from one state to no more than one other.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct State<I: Clone + Ord> {
    /// Transitions that require consuming and matching input.
    pub(crate) transitions: BTreeMap<I, usize>,
    /// Whether an input that ends in this state ought to be accepted.
    pub(crate) accepting: bool,
}

impl<I: Clone + Ord> Graph<I> {
    /// Get the state at a given index.
    #[must_use]
    #[inline(always)]
    pub fn get(&self, i: usize) -> Option<&State<I>> {
        self.states.get(i)
    }

    /// Decide whether an input belongs to the regular langage this NFA accepts.
    #[inline(always)]
    #[allow(clippy::missing_panics_doc)]
    pub fn accept<Iter: IntoIterator<Item = I>>(&self, iter: Iter) -> bool {
        if self.states.is_empty() {
            return false;
        }
        let mut state = self.initial;
        for input in iter {
            match get!(self.states, state).transition(&input) {
                Some(&next_state) => state = next_state,
                None => return false,
            }
        }
        get!(self.states, state).is_accepting()
    }

    /// DFA with zero states.
    #[must_use]
    #[inline(always)]
    pub const fn invalid() -> Self {
        Self {
            states: vec![],
            initial: usize::MAX,
        }
    }

    /// Check if there are any states (empty would be illegal, but hey, why crash your program).
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    /// Number of states.
    #[must_use]
    #[inline(always)]
    pub fn size(&self) -> usize {
        self.states.len()
    }
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

impl<I: Clone + Ord + core::fmt::Display> core::fmt::Display for Graph<I> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Initial state: {}", self.initial)?;
        for (i, state) in self.states.iter().enumerate() {
            write!(f, "State {i} {state}")?;
        }
        Ok(())
    }
}

impl<I: Clone + Ord + core::fmt::Display> core::fmt::Display for State<I> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(
            f,
            "({}accepting):",
            if self.is_accepting() { "" } else { "NOT " }
        )?;
        for (input, transitions) in &self.transitions {
            writeln!(f, "    {input} --> {transitions}")?;
        }
        Ok(())
    }
}

impl<I: Clone + Ord> State<I> {
    /// State to which this state can transition on a given input.
    #[inline]
    pub fn transition(&self, input: &I) -> Option<&usize> {
        self.transitions.get(input)
    }

    /// Whether an input that ends in this state ought to be accepted.
    #[inline(always)]
    pub const fn is_accepting(&self) -> bool {
        self.accepting
    }
}

#[cfg(feature = "quickcheck")]
impl<I: Ord + quickcheck::Arbitrary> quickcheck::Arbitrary for Graph<I> {
    #[inline]
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let mut states = quickcheck::Arbitrary::arbitrary(g);
        cut_nonsense(&mut states);
        let size = states.len();
        Self {
            states,
            initial: usize::arbitrary(g).checked_rem(size).unwrap_or(0),
        }
    }

    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.states.clone(), self.initial)
                .shrink()
                .map(|(mut states, initial)| {
                    cut_nonsense(&mut states);
                    let size = states.len();
                    Self {
                        states,
                        initial: initial.checked_rem(size).unwrap_or(0),
                    }
                }),
        )
    }
}

#[cfg(feature = "quickcheck")]
impl<I: Ord + quickcheck::Arbitrary> quickcheck::Arbitrary for State<I> {
    #[inline]
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self {
            transitions: quickcheck::Arbitrary::arbitrary(g),
            accepting: quickcheck::Arbitrary::arbitrary(g),
        }
    }

    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new((self.transitions.clone(), self.accepting).shrink().map(
            |(transitions, accepting)| Self {
                transitions,
                accepting,
            },
        ))
    }
}

/// Remove impossible transitions from automatically generated automata.
#[cfg(feature = "quickcheck")]
fn cut_nonsense<I: Clone + Ord>(v: &mut Vec<State<I>>) {
    let size = v.len();
    for state in v {
        state.transitions.retain(|_, index| *index < size);
    }
}
