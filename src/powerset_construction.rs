/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! The powerset construction algorithm for constructing an equivalent DFA from an arbitrary NFA.
//! Also known as the subset construction algorithm.

use crate::{call::Call, dfa, nfa, range::Range, Compiled as Dfa, Expression, Parser as Nfa};
use core::fmt::Debug;
use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};

/// Subset of states (by their index).
type Subset = BTreeSet<usize>;

/// From a single state, all tokens and the transitions each would induce.
type Transitions<I> = BTreeMap<Range<I>, Transition<I>>;

/// A single edge triggered by a token.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Transition<I: Clone + Ord> {
    /// Set of destination states.
    dsts: Subset,
    /// Function (or none) to call on this edge.
    call: Call,
    /// Index of the state this transition *leaves*.
    from_state: usize,
    /// Token that triggers this transition.
    on_token: Range<I>,
    /// Minimal reproducible input string to reach this transition.
    breadcrumbs: Vec<I>,
}

/// A collection of outgoing edges and a boolean to mark accepting states.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct SubsetState<I: Clone + Ord> {
    /// Transitions from this subset of states to other subsets on certain tokens.
    transitions: Transitions<I>,
    /// Whether we should accept a string that ends in this state.
    accepting: Option<Call>,
}

/// Map each _subset_ of NFA states to a future DFA state.
type SubsetsAsStates<I> = BTreeMap<Subset, SubsetState<I>>;

/// Postpone a call from an original call site to some set of later states.
#[allow(dead_code)] // <-- FIXME
struct Postpone<I: Clone + Expression + Ord> {
    /// Origin of the transition on the original graph.
    l_state: usize,
    /// Token that generates an ambiguous transition.
    l_token: Range<I>,
    /// Sequence of tokens that could lead to `l_state`.
    l_so_far: Vec<I>,
    /// Origin of the transition on the original graph.
    r_state: usize,
    /// Token that generates an ambiguous transition.
    r_token: Range<I>,
    /// Sequence of tokens that could lead to `r_state`.
    r_so_far: Vec<I>,
    /// Whether to save the value of this token for later.
    stash: bool,
    /// Left call (not really "left" in any meaningful sense, but the first one).
    pl: Call,
    /// Right call (not really "right" in any meaningful sense, but the second one).
    pr: Call,
    /// Subset of states that will now be responsible for the call.
    to: Subset,
}

impl<I: Clone + Expression + Ord + Debug> Nfa<I> {
    /// Powerset construction algorithm mapping subsets of states to DFA nodes.
    #[inline]
    pub(crate) fn subsets(mut self) -> Dfa<I> {
        // Map which _subsets_ of states transition to which _subsets_ of states
        let (subsets_as_states, initial_state) = self.postpone_ambiguities();

        // Fix an ordering on subsets so each can be a DFA state
        let mut ordered: Vec<_> = subsets_as_states.keys().collect();
        ordered.sort_unstable();

        // Check that binary_search works
        #[cfg(test)]
        {
            for (i, subset) in ordered.iter().enumerate() {
                assert_eq!(ordered.binary_search(subset), Ok(i));
            }
        }

        // Construct the vector of subset-mapped states
        let states = ordered
            .iter()
            .map(|&subset| {
                let SubsetState {
                    ref transitions,
                    ref accepting,
                } = *unwrap!(subsets_as_states.get(subset));
                dfa::State {
                    transitions: transitions
                        .iter()
                        .map(
                            |(
                                token,
                                &Transition {
                                    ref dsts, ref call, ..
                                },
                            )| {
                                (
                                    token.clone(),
                                    dfa::Transition {
                                        dst: unwrap!(ordered.binary_search(&dsts)),
                                        call: call.clone(),
                                    },
                                )
                            },
                        )
                        .collect(),
                    accepting: accepting.clone(),
                }
            })
            .collect();

        // Wrap it in a DFA
        Dfa {
            states,
            initial: unwrap!(ordered.binary_search(&&initial_state)),
        }
    }

    /// Postpone calls that would be ambiguous until they're no longer ambiguous (if ever).
    #[inline]
    #[allow(clippy::panic)]
    fn postpone_ambiguities(&mut self) -> (SubsetsAsStates<I>, Subset) {
        // #[allow(clippy::default_numeric_fallback)]
        // for _ in 0..(1_usize << self.states.len()) {
        loop {
            let pre = self.clone();
            let mut subsets_as_states = SubsetsAsStates::new();
            match self.traverse(
                self.initial.iter().copied().collect(),
                &mut subsets_as_states,
                vec![],
            ) {
                Ok(subset) => return (subsets_as_states, subset),
                Err(Postpone {
                    l_state: l_idx,
                    ref l_token,
                    l_so_far,
                    r_state: r_idx,
                    ref r_token,
                    r_so_far,
                    stash,
                    pl,
                    pr,
                    ..
                }) => {
                    let replace = if stash { Call::Stash } else { Call::Pass };
                    unwrap!(get_mut!(self.states, l_idx).non_epsilon.get_mut(l_token)).call =
                        replace.clone();
                    unwrap!(get_mut!(self.states, r_idx).non_epsilon.get_mut(r_token)).call =
                        replace;
                    #[allow(clippy::print_stdout)]
                    {
                        println!("Postponing ({pl:?}, {pr:?}) from (#{l_idx} on {l_token:?}, #{r_idx} on {r_token:?}). So far: {l_so_far:?}");
                    }
                    self.postpone(l_idx, &pl, l_so_far, l_token);
                    self.postpone(r_idx, &pr, r_so_far, r_token);
                }
            }
            assert_ne!(
                *self, pre,
                "INTERNAL ERROR: \
                Ran a postponement step but nothing changed. \
                Please report!",
            );
        }
        // panic!("Timed out");
    }

    /// Postpone a call that's already been removed to the set of states that could follow the one it was removed form.
    fn postpone(&mut self, idx: usize, call: &Call, mut so_far: Vec<I>, token: &Range<I>) {
        let mut did_anything = false;
        let mut neighbors: Vec<usize> = self
            .take_all_epsilon_transitions(vec![idx])
            .into_iter()
            .flat_map(|i| {
                self.take_all_epsilon_transitions(
                    get!(self.states, i)
                        .non_epsilon
                        .values()
                        .flat_map(|&nfa::Transition { ref dsts, .. }| dsts.iter().copied())
                        .collect(),
                )
            })
            .collect();
        neighbors.sort_unstable();
        neighbors.dedup();
        for &i in &neighbors {
            get_mut!(self.states, i).take_responsibility_for(call, &mut did_anything);
        }
        assert!(
            did_anything,
            "Parsing ambiguity after [{}] on token {token:?}: \
            can't immediately decide between TODO and TODO, \
            but [EXPLANATION TODO]. \
            (Internally, we couldn't postpone `{call:?}` \
            from {idx} to {neighbors:?} \
            because there's nowhere for it to go.)",
            so_far.pop().map_or_else(String::new, |last| {
                so_far
                    .iter()
                    .fold(String::new(), |acc, i| acc + &format!("{i:?} -> "))
                    + &format!("{last:?}")
            }),
        );
    }

    /// Map which _subsets_ of states transition to which _subsets_ of states.
    /// Return the expansion of the original `queue` argument after taking all epsilon transitions.
    #[inline]
    #[allow(clippy::todo)] // <-- FIXME
    #[allow(
        clippy::panic_in_result_fn,
        clippy::too_many_lines,
        clippy::unwrap_in_result
    )]
    fn traverse(
        &self,
        queue: Vec<usize>,
        subsets_as_states: &mut SubsetsAsStates<I>,
        mut so_far: Vec<I>,
    ) -> Result<Subset, Postpone<I>> // <-- Return the set of states after taking epsilon transitions
    {
        // Take all epsilon transitions immediately
        let post_epsilon = self.take_all_epsilon_transitions(queue);

        // Check if we've already seen this subset
        let tmp = match subsets_as_states.entry(post_epsilon.clone()) {
            Entry::Occupied(_) => return Ok(post_epsilon),
            Entry::Vacant(empty) => empty,
        };

        // For now, so we can't get stuck in a cycle, cache an empty map
        #[allow(clippy::panic)]
        let _ = tmp.insert(SubsetState {
            transitions: BTreeMap::new(),
            accepting: post_epsilon.iter().fold(None, |acc, &i| {
                match (acc, get!(self.states, i).accepting.clone()) {
                    (None, None) => None,
                    (Some(x), None) | (None, Some(x)) => Some(x),
                    (Some(a), Some(b)) if a == b => Some(a),
                    (Some(a), Some(b)) => panic!(
                        "Parsing ambiguity after [{}] if input ends here: \
                        can't decide between {} and {}.",
                        so_far.pop().map_or_else(String::new, |last| {
                            so_far
                                .iter()
                                .fold(String::new(), |s, j| s + &format!("{j:?} -> "))
                                + &format!("{last:?}")
                        }),
                        a.verbal(),
                        b.verbal(),
                    ),
                }
            }),
        });

        // Calculate all non-epsilon transitions out of this *subset* of states,
        // converted into transitions out of a *single* DFA state.
        let mut transitions = Transitions::<I>::new();
        // For each state we're currently inhabiting,...
        for &index in &post_epsilon {
            let state = get!(self.states, index);

            // ... for each non-epsilon transition out of that state...
            for (
                token,
                &nfa::Transition {
                    dsts: ref map,
                    call: ref new_call,
                },
            ) in &state.non_epsilon
            {
                // ... compute the input string that reached this state ...
                let mut input_string = so_far.clone();
                input_string.push(token.start.clone());
                // ... check if we already have any other transitions on that token.
                match transitions.entry(token.clone()) {
                    // If we don't, ...
                    Entry::Vacant(entry) => {
                        // ... insert a new transition.
                        let _ = entry.insert(Transition {
                            dsts: map.clone(),
                            call: new_call.clone(),
                            from_state: index,
                            on_token: token.clone(),
                            breadcrumbs: input_string,
                        });
                    }
                    // If we already have a transition on this token, ...
                    Entry::Occupied(entry) => {
                        // ... check what we have so far, ...
                        let &mut Transition {
                            ref mut dsts,
                            call: ref mut existing_call,
                            ref from_state,
                            ref on_token,
                            ref mut breadcrumbs,
                        } = entry.into_mut();
                        // ... and if we have a shorter input string that reached here, replace it.
                        if input_string.len() < breadcrumbs.len() {
                            *breadcrumbs = input_string;
                        }
                        // Next, check for trying to call two different functions on the same input.
                        #[allow(clippy::panic)]
                        let compatible = new_call
                            .clone()
                            .compat(existing_call.clone())
                            .unwrap_or_else(|| {
                                panic!(
                                    "Parsing ambiguity after [{}] on token {token:?}: \
                                    can't immediately decide between {} and {}, \
                                    but a function reads this exact token at runtime. \
                                    (Note: if you rewrite the function without \
                                    reading the value of the token it parsed, \
                                    we can automatically postpone the decision \
                                    until some later token is differ[<43;55;7M[<43;56;7Ment.)",
                                    so_far.pop().map_or_else(String::new, |last| {
                                        so_far.iter().fold(String::new(), |acc, i| {
                                            acc + &format!("{i:?} -> ")
                                        }) + &format!("{last:?}")
                                    }),
                                    new_call.verbal(),
                                    existing_call.verbal(),
                                )
                            });
                        match compatible {
                            // If it's the same function, ...
                            Ok(common) => {
                                // ... then just add some states to the subset.
                                dsts.extend(map.iter().copied());
                                *existing_call = common;
                            }
                            // If not identical but still compatible, ...
                            Err((stash, pl, pr)) => {
                                // ... and NOT an accepting state (in which case postponing might mean we never call it), ...
                                assert_eq!(
                                    None,
                                    state.accepting,
                                    "Parsing ambiguity after [{}] on token {token:?}: \
                                    can't immediately decide between {} and {}, \
                                    but this state is accepting, so we can't \
                                    postpone until a difference is observed.",
                                    so_far.pop().map_or_else(String::new, |last| {
                                        so_far.iter().fold(String::new(), |acc, i| {
                                            acc + &format!("{i:?} -> ")
                                        }) + &format!("{last:?}")
                                    }),
                                    new_call.verbal(),
                                    existing_call.verbal(),
                                );
                                // ... postpone the call to the next subset of states.
                                return Err(Postpone {
                                    l_state: index,
                                    l_token: token.clone(),
                                    l_so_far: so_far.clone(),
                                    r_state: *from_state,
                                    r_token: on_token.clone(),
                                    r_so_far: so_far.clone(),
                                    stash,
                                    pl,
                                    pr,
                                    to: dsts.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }

        // Now, follow epsilon transitions AND recurse
        for &mut Transition {
            ref mut dsts,
            ref mut breadcrumbs,
            ..
        } in transitions.values_mut()
        {
            *dsts = self.traverse(
                dsts.iter().copied().collect(),
                subsets_as_states,
                breadcrumbs.clone(),
            )?;
        }

        // Rewrite the empty map we wrote earlier with the actual transitions
        unwrap!(subsets_as_states.get_mut(&post_epsilon)).transitions = transitions;

        Ok(post_epsilon)
    }
}

impl<I: Clone + Expression + Ord> nfa::State<I> {
    /// Check if this state is not accepting AND has no outgoing transitions.
    #[allow(dead_code)] // <-- FIXME
    fn is_dead_end(&self) -> bool {
        self.accepting.is_none() && self.non_epsilon.is_empty()
    }

    /// Receive a postponed call and implement it here.
    #[inline]
    #[allow(dead_code)] // <-- FIXME
    fn take_responsibility_for(&mut self, call: &Call, did_anything: &mut bool) {
        if let Some(ref mut accept_fn) = self.accepting {
            accept_fn.take_responsibility_for(call.clone(), did_anything);
        }
        for transition in self.non_epsilon.values_mut() {
            transition
                .call
                .take_responsibility_for(call.clone(), did_anything);
            //        ^^^^^ crucial that we call `Call::take_resp...` rather than `State::take_resp...` to avoid infinite recursion
        }
    }
}

impl Call {
    /// Receive a postponed call and implement it here.
    #[inline]
    #[allow(clippy::panic)] // <-- TODO
    fn take_responsibility_for(&mut self, call: Call, did_anything: &mut bool) {
        *did_anything = true;
        if call == Self::Pass || *self == call {
            return;
        }
        match *self {
            Self::Pass => *self = call,
            Self::Stash => panic!(
                "Not yet implemented. \
                Please open a pull request with the \
                conditions that led to this error \
                and a copy of the following: \
                \"({self:?}, {call:?})\"",
            ),
            Self::WithToken(ref mut stack) | Self::WithoutToken(ref mut stack) => match call {
                Self::Pass => {}
                Self::Stash => panic!(
                    "Not yet implemented. \
                    Please open a pull request with the \
                    conditions that led to this error \
                    and a copy of the following: \
                    \"({self:?}, {call:?})\"",
                ),
                Self::WithToken(other) | Self::WithoutToken(other) => stack.extend(other),
            },
        }
    }
}
