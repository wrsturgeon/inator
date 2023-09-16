use std::collections::BTreeMap;

/// Deterministic finite automata.
#[repr(transparent)]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Graph<I: Clone + Ord> {
    states: Vec<State<I>>,
}

impl<I: Clone + Ord> Graph<I> {
    #[inline(always)]
    pub const fn new(states: Vec<State<I>>) -> Self {
        Self { states }
    }

    #[inline(always)]
    pub fn get(&self, i: usize) -> Option<&State<I>> {
        self.states.get(i)
    }

    #[inline(always)]
    pub fn accept<Iter: IntoIterator<Item = I>>(&self, iter: Iter) -> bool {
        if self.states.len() == 0 {
            return false;
        }
        let mut state = 0;
        for input in iter {
            match unwrap!(self.get(state)).transition(&input) {
                Some(&next_state) => state = next_state,
                None => return false,
            }
        }
        unwrap!(self.get(state)).is_accepting()
    }
}

/// State transitions from one state to no more than one other.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct State<I: Clone + Ord> {
    transitions: BTreeMap<I, usize>,
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

impl<I: Clone + Ord> State<I> {
    #[inline]
    pub fn transition(&self, input: &I) -> Option<&usize> {
        self.transitions.get(input)
    }

    #[inline(always)]
    pub const fn is_accepting(&self) -> bool {
        self.accepting
    }
}

impl<I: Clone + Ord> From<BTreeMap<I, usize>> for State<I> {
    #[inline(always)]
    fn from(transitions: BTreeMap<I, usize>) -> Self {
        Self {
            transitions,
            accepting: todo!(),
        }
    }
}
