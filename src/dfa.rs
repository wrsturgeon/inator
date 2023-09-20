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
    /// Every state in this graph (should never be empty!).
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
        let mut state = self.initial;
        for input in iter {
            match get!(self.states, state).transition(&input) {
                Some(&next_state) => state = next_state,
                None => return false,
            }
        }
        get!(self.states, state).accepting
    }

    /// Number of states.
    #[must_use]
    #[inline(always)]
    pub fn size(&self) -> usize {
        self.states.len()
    }

    /// Generalize to an identical NFA.
    #[inline]
    #[must_use]
    pub fn generalize(&self) -> crate::Nfa<I> {
        crate::Nfa {
            states: self
                .states
                .iter()
                .map(|state| crate::nfa::State {
                    epsilon: std::collections::BTreeSet::new(),
                    non_epsilon: state
                        .transitions
                        .iter()
                        .map(|(k, &v)| (k.clone(), core::iter::once(v).collect()))
                        .collect(),
                    accepting: state.accepting,
                })
                .collect(),
            initial: core::iter::once(self.initial).collect(),
        }
    }

    /// Randomly generate inputs that are all guaranteed to be accepted.
    /// NOTE: returns an infinite iterator! `for input in automaton.fuzz()?` will loop forever . . .
    /// # Errors
    /// If this automaton never accepts any input.
    #[inline]
    pub fn fuzz(&self) -> Result<crate::Fuzzer<I>, crate::NeverAccepts> {
        self.generalize().fuzz()
    }

    /// Check if there exists a string this DFA will accept.
    #[inline]
    #[must_use]
    pub fn would_ever_accept(&self) -> bool {
        self.states.iter().any(|state| state.accepting)
    }
}

impl Graph<crate::expr::Expression> {
    /// Print as a set of Rust source-code functions.
    #[inline]
    #[must_use]
    #[allow(clippy::too_many_lines)]
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
        sig.output = syn::ReturnType::Type(Token!(->)(Span::call_site()), Box::new(out_t.clone()));
        let generics = sig.generics.clone();
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
            self.states
                .iter()
                .enumerate()
                .map(move |(i, state)| state.as_source(i, generics.clone(), out_t.clone()))
                .collect(),
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
            if self.accepting { "" } else { "NOT " }
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
}

impl State<crate::expr::Expression> {
    /// Print as a Rust source-code function.
    #[inline]
    #[allow(clippy::too_many_lines)]
    pub fn as_source(&self, index: usize, generics: syn::Generics, out_t: syn::Type) -> syn::Item {
        syn::Item::Fn(syn::ItemFn {
            attrs: vec![syn::Attribute {
                pound_token: Token!(#)(Span::call_site()),
                style: syn::AttrStyle::Outer,
                bracket_token: syn::token::Bracket::default(),
                meta: syn::Meta::Path(syn::Path {
                    leading_colon: None,
                    segments: core::iter::once(syn::PathSegment {
                        ident: Ident::new("inline", Span::call_site()),
                        arguments: syn::PathArguments::None,
                    })
                    .collect(),
                }),
            }],
            vis: syn::Visibility::Public(Token!(pub)(Span::call_site())),
            sig: syn::Signature {
                constness: None,
                asyncness: None,
                unsafety: None,
                abi: None,
                fn_token: Token!(fn)(Span::call_site()),
                ident: Ident::new(&format!("s{index}"), Span::call_site()),
                generics,
                paren_token: syn::token::Paren::default(),
                inputs: core::iter::once(syn::FnArg::Typed(syn::PatType {
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
                .collect(),
                variadic: None,
                output: syn::ReturnType::Type(
                    Token!(->)(Span::call_site()),
                    Box::new(syn::Type::Path(syn::TypePath {
                        qself: None,
                        path: syn::Path {
                            leading_colon: None,
                            segments: core::iter::once(syn::PathSegment {
                                ident: Ident::new("Result", Span::call_site()),
                                arguments: syn::PathArguments::AngleBracketed(
                                    syn::AngleBracketedGenericArguments {
                                        colon2_token: None,
                                        lt_token: Token!(<)(Span::call_site()),
                                        args: core::iter::once(syn::GenericArgument::Type(out_t))
                                            .collect(),
                                        gt_token: Token!(>)(Span::call_site()),
                                    },
                                ),
                            })
                            .collect(),
                        },
                    })),
                ),
            },
            block: Box::new(syn::Block {
                brace_token: syn::token::Brace::default(),
                stmts: vec![syn::Stmt::Expr(
                    syn::Expr::Match(syn::ExprMatch {
                        attrs: vec![],
                        match_token: Token!(match)(Span::call_site()),
                        expr: Box::new(syn::Expr::MethodCall(syn::ExprMethodCall {
                            attrs: vec![],
                            receiver: Box::new(syn::Expr::Path(syn::ExprPath {
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
                            })),
                            dot_token: Token!(.)(Span::call_site()),
                            method: Ident::new("next", Span::call_site()),
                            turbofish: None,
                            paren_token: syn::token::Paren::default(),
                            args: syn::punctuated::Punctuated::new(),
                        })),
                        brace_token: syn::token::Brace::default(),
                        arms: self
                            .transitions
                            .iter()
                            .map(|(input, dst)| syn::Arm {
                                attrs: vec![],
                                pat: syn::Pat::TupleStruct(syn::PatTupleStruct {
                                    attrs: vec![],
                                    qself: None,
                                    path: syn::Path {
                                        leading_colon: None,
                                        segments: core::iter::once(syn::PathSegment {
                                            ident: Ident::new("Some", Span::call_site()),
                                            arguments: syn::PathArguments::None,
                                        })
                                        .collect(),
                                    },
                                    paren_token: syn::token::Paren::default(),
                                    elems: core::iter::once(input.as_pattern()).collect(),
                                }),
                                guard: None,
                                fat_arrow_token: Token!(=>)(Span::call_site()),
                                body: Box::new(
                                    // syn::Expr::Call(syn::ExprCall {
                                    //     attrs: vec![],
                                    //     func: Box::new(syn::Expr::Path(syn::ExprPath {
                                    //         attrs: vec![],
                                    //         qself: None,
                                    //         path: syn::Path {
                                    //             leading_colon: None,
                                    //             segments: core::iter::once(syn::PathSegment {
                                    //                 ident: Ident::new("Ok", Span::call_site()),
                                    //                 arguments: syn::PathArguments::None,
                                    //             })
                                    //             .collect(),
                                    //         },
                                    //     })),
                                    //     paren_token: syn::token::Paren::default(),
                                    //     args: core::iter::once(
                                    syn::Expr::Call(syn::ExprCall {
                                        attrs: vec![],
                                        func: Box::new(syn::Expr::Path(syn::ExprPath {
                                            attrs: vec![],
                                            qself: None,
                                            path: syn::Path {
                                                leading_colon: None,
                                                segments: core::iter::once(syn::PathSegment {
                                                    ident: Ident::new(
                                                        &format!("s{dst}"),
                                                        Span::call_site(),
                                                    ),
                                                    arguments: syn::PathArguments::None,
                                                })
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
                                    }), //     )
                                        //     .collect(),
                                        // })
                                ),
                                comma: Some(Token!(,)(Span::call_site())),
                            })
                            .chain(core::iter::once(syn::Arm {
                                attrs: vec![],
                                pat: syn::Pat::Path(syn::ExprPath {
                                    attrs: vec![],
                                    qself: None,
                                    path: syn::Path {
                                        leading_colon: None,
                                        segments: core::iter::once(syn::PathSegment {
                                            ident: Ident::new("None", Span::call_site()),
                                            arguments: syn::PathArguments::None,
                                        })
                                        .collect(),
                                    },
                                }),
                                guard: None,
                                fat_arrow_token: Token!(=>)(Span::call_site()),
                                body: Box::new(if self.accepting {
                                    syn::Expr::Call(syn::ExprCall {
                                        attrs: vec![],
                                        func: Box::new(syn::Expr::Path(syn::ExprPath {
                                            attrs: vec![],
                                            qself: None,
                                            path: syn::Path {
                                                leading_colon: None,
                                                segments: core::iter::once(syn::PathSegment {
                                                    ident: Ident::new("Ok", Span::call_site()),
                                                    arguments: syn::PathArguments::None,
                                                })
                                                .collect(),
                                            },
                                        })),
                                        paren_token: syn::token::Paren::default(),
                                        args: core::iter::once(syn::Expr::Tuple(syn::ExprTuple {
                                            attrs: vec![],
                                            paren_token: syn::token::Paren::default(),
                                            elems: syn::punctuated::Punctuated::new(),
                                        }))
                                        .collect(),
                                    })
                                } else {
                                    syn::Expr::Call(syn::ExprCall {
                                        attrs: vec![],
                                        func: Box::new(syn::Expr::Path(syn::ExprPath {
                                            attrs: vec![],
                                            qself: None,
                                            path: syn::Path {
                                                leading_colon: None,
                                                segments: core::iter::once(syn::PathSegment {
                                                    ident: Ident::new("Err", Span::call_site()),
                                                    arguments: syn::PathArguments::None,
                                                })
                                                .collect(),
                                            },
                                        })),
                                        paren_token: syn::token::Paren::default(),
                                        args: syn::punctuated::Punctuated::new(),
                                    })
                                }),
                                comma: Some(Token!(,)(Span::call_site())),
                            }))
                            .chain(core::iter::once(syn::Arm {
                                attrs: vec![],
                                pat: syn::Pat::Wild(syn::PatWild {
                                    attrs: vec![],
                                    underscore_token: Token!(_)(Span::call_site()),
                                }),
                                guard: None,
                                fat_arrow_token: Token!(=>)(Span::call_site()),
                                body: Box::new(syn::Expr::Call(syn::ExprCall {
                                    attrs: vec![],
                                    func: Box::new(syn::Expr::Path(syn::ExprPath {
                                        attrs: vec![],
                                        qself: None,
                                        path: syn::Path {
                                            leading_colon: None,
                                            segments: core::iter::once(syn::PathSegment {
                                                ident: Ident::new("Err", Span::call_site()),
                                                arguments: syn::PathArguments::None,
                                            })
                                            .collect(),
                                        },
                                    })),
                                    paren_token: syn::token::Paren::default(),
                                    args: syn::punctuated::Punctuated::new(),
                                })),
                                comma: Some(Token!(,)(Span::call_site())),
                            }))
                            .collect(),
                    }),
                    None,
                )],
            }),
        })
    }
}

#[cfg(feature = "quickcheck")]
impl<I: Ord + quickcheck::Arbitrary> quickcheck::Arbitrary for Graph<I> {
    #[inline]
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let mut states = Vec::arbitrary(g);
        while states.is_empty() {
            states = Vec::arbitrary(g);
        }
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
