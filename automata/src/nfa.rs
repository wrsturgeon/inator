use std::collections::{BTreeMap, BTreeSet};

/// Nondeterministic finite automata allowing epsilon transitions.
#[repr(transparent)]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Graph<I: Clone + Ord> {
    states: Vec<State<I>>,
}

/// Transitions from one state to arbitrarily many others, possibly without even consuming input.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct State<I: Clone + Ord> {
    epsilon: BTreeSet<usize>,
    non_epsilon: BTreeMap<I, BTreeSet<usize>>,
    accepting: bool,
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
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    #[inline(always)]
    pub fn get(&self, i: usize) -> Option<&State<I>> {
        self.states.get(i)
    }

    #[inline]
    pub fn take_all_epsilon_transitions(&self, mut queue: BTreeSet<usize>) -> BTreeSet<usize> {
        // Take all epsilon transitions immediately
        let mut superposition = BTreeSet::<usize>::new();
        while let Some(state) = queue.pop_first() {
            for next in unwrap!(self.get(state)).epsilon_transitions() {
                if !superposition.contains(next) {
                    queue.insert(*next);
                }
            }
            superposition.insert(state);
        }
        superposition
    }

    #[inline]
    pub fn accept<Iter: IntoIterator<Item = I>>(&self, iter: Iter) -> bool {
        if self.states.len() == 0 {
            return false;
        }
        let mut state = core::iter::once(0).collect();
        for input in iter {
            state = self.take_all_epsilon_transitions(state);
            let mut new_state = BTreeSet::new();
            for index in state {
                let Some(transitions) = unwrap!(self.get(index)).transition(&input) else {
                    return false;
                };
                new_state.extend(transitions);
            }
            state = new_state;
        }
        state
            .into_iter()
            .any(|index| unwrap!(self.get(index)).is_accepting())
    }
}

impl<I: Clone + Ord> State<I> {
    #[inline(always)]
    pub fn non_epsilon_transitions(&self) -> &BTreeMap<I, BTreeSet<usize>> {
        &self.non_epsilon
    }

    #[inline(always)]
    pub fn epsilon_transitions(&self) -> &BTreeSet<usize> {
        &self.epsilon
    }

    #[inline]
    pub fn transition(&self, input: &I) -> Option<&BTreeSet<usize>> {
        self.non_epsilon.get(input)
    }

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
        Self { states }
    }

    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(self.states.shrink().map(|mut states| {
            cut_nonsense(&mut states);
            Self { states }
        }))
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

// #[cfg(feature = "quickcheck")]
fn cut_nonsense<I: Clone + Ord>(v: &mut Vec<State<I>>) {
    let size = v.len();
    for state in v {
        state.epsilon.retain(|i| i < &size);
        for v in state.non_epsilon.values_mut() {
            v.retain(|i| i < &size);
        }
    }
}
