/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Deterministic finite automata.

use proc_macro2::Span;
use std::collections::BTreeMap;
use syn::{Ident, Token, __private::ToTokens};

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

    /// Print as a set of Rust source-code functions.
    #[inline]
    pub fn as_source(
        &self,
        f: &syn::ItemFn,
        in_t: syn::Type,
        out_t: syn::Type,
    ) -> (syn::ItemFn, Vec<syn::Item>) {
        let mut sig = f.sig.clone();
        sig.generics
            .params
            .push(syn::GenericParam::Type(syn::TypeParam {
                attrs: vec![],
                ident: Ident::new("I", Span::call_site()),
                colon_token: Some(Token!(:)(Span::call_site())),
                bounds: core::iter::once(syn::TypeParamBound::Trait(syn::TraitBound {
                    paren_token: Some(syn::token::Paren::default()),
                    modifier: syn::TraitBoundModifier::None,
                    lifetimes: None,
                    path: syn::Path {
                        leading_colon: None,
                        segments: core::iter::once(syn::PathSegment {
                            ident: Ident::new("Iterator", Span::call_site()),
                            arguments: syn::PathArguments::AngleBracketed(
                                syn::AngleBracketedGenericArguments {
                                    colon2_token: None,
                                    lt_token: Token!(<)(Span::call_site()),
                                    args: core::iter::once(syn::GenericArgument::AssocType(
                                        syn::AssocType {
                                            ident: Ident::new("Item", Span::call_site()),
                                            generics: None,
                                            eq_token: Token!(=)(Span::call_site()),
                                            ty: in_t,
                                        },
                                    ))
                                    .collect(),
                                    gt_token: Token!(>)(Span::call_site()),
                                },
                            ),
                        })
                        .collect(),
                    },
                }))
                .collect(),
                eq_token: None,
                default: None,
            }));
        sig.inputs = core::iter::once(syn::FnArg::Typed(syn::PatType {
            attrs: vec![],
            pat: Box::new(syn::Pat::Ident(syn::PatIdent {
                attrs: vec![],
                by_ref: None,
                mutability: None,
                ident: Ident::new("i", Span::call_site()),
                subpat: None,
            })),
            colon_token: Token!(:)(Span::call_site()),
            ty: Box::new(syn::Type::Path(syn::TypePath {
                qself: None,
                path: syn::Path {
                    leading_colon: None,
                    segments: core::iter::once(syn::PathSegment {
                        ident: Ident::new("I", Span::call_site()),
                        arguments: syn::PathArguments::None,
                    })
                    .collect(),
                },
            })),
        }))
        .collect();
        sig.output = syn::ReturnType::Type(Token!(->)(Span::call_site()), Box::new(out_t));
        (
            syn::ItemFn {
                attrs: vec![syn::Attribute {
                    pound_token: Token!(#)(Span::call_site()),
                    style: syn::AttrStyle::Outer,
                    bracket_token: syn::token::Bracket::default(),
                    meta: syn::Meta::List(syn::MetaList {
                        path: syn::Path {
                            leading_colon: None,
                            segments: core::iter::once(syn::PathSegment {
                                ident: Ident::new("inline", Span::call_site()),
                                arguments: syn::PathArguments::None,
                            })
                            .collect(),
                        },
                        delimiter: syn::MacroDelimiter::Paren(syn::token::Paren::default()),
                        tokens: Ident::new("always", Span::call_site()).into_token_stream(),
                    }),
                }],
                block: Box::new(syn::Block {
                    brace_token: syn::token::Brace::default(),
                    stmts: vec![syn::Stmt::Expr(
                        syn::Expr::Call(syn::ExprCall {
                            attrs: vec![],
                            func: Box::new(syn::Expr::Path(syn::ExprPath {
                                attrs: vec![],
                                qself: None,
                                path: syn::Path {
                                    leading_colon: None,
                                    segments: [
                                        syn::PathSegment {
                                            ident: Ident::new(
                                                &format!("_inator_automaton_{}", f.sig.ident),
                                                Span::call_site(),
                                            ),
                                            arguments: syn::PathArguments::None,
                                        },
                                        syn::PathSegment {
                                            ident: Ident::new(
                                                &format!("s{}", self.initial),
                                                Span::call_site(),
                                            ),
                                            arguments: syn::PathArguments::None,
                                        },
                                    ]
                                    .into_iter()
                                    .collect(),
                                },
                            })),
                            paren_token: syn::token::Paren::default(),
                            args: core::iter::once(syn::Expr::Path(syn::ExprPath {
                                attrs: vec![],
                                qself: None,
                                path: syn::Path {
                                    leading_colon: None,
                                    segments: core::iter::once(syn::PathSegment {
                                        ident: Ident::new("i", Span::call_site()),
                                        arguments: syn::PathArguments::None,
                                    })
                                    .collect(),
                                },
                            }))
                            .collect(),
                        }),
                        None,
                    )],
                }),
                sig,
                vis: syn::Visibility::Inherited,
            },
            vec![], // TODO
                    // self.states
                    //     .iter()
                    //     .enumerate()
                    //     .map(|(i, state)| state.as_source(i, &in_t, &out_t)),
                    //     .collect()
        )
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

    /// Print as a Rust source-code function.
    #[inline]
    pub fn as_source(&self, _index: usize, _in_t: &str, _out_t: &str) -> syn::ItemFn {
        // let mut s = format!("#[inline]fn s{index}<I:Iterator<Item={in_t}>>(i:I)->{out_t}{{");
        // s.push_str("todo!()"); // TODO
        // s.push('}');
        // s
        todo!()
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
