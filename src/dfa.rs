/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Deterministic finite automata.

use crate::Expression;
use proc_macro2::Span;
use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};
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
    pub(crate) transitions: BTreeMap<I, (usize, Option<&'static str>)>,
    /// Whether an input that ends in this state ought to be accepted.
    pub(crate) accepting: bool,
}

impl<I: Clone + Ord> Graph<I> {
    /// Decide whether an input belongs to the regular langage this NFA accepts.
    #[inline]
    #[allow(clippy::missing_panics_doc)]
    pub fn accept<Iter: IntoIterator<Item = I>>(&self, iter: Iter) -> bool {
        let mut state = self.initial;
        for input in iter {
            match get!(self.states, state).transition(&input) {
                Some(&(next_state, _)) => state = next_state,
                None => return false,
            }
        }
        get!(self.states, state).accepting
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

    /// Generalize to an identical NFA.
    #[inline]
    #[must_use]
    pub fn generalize(&self) -> crate::nfa::Graph<I> {
        crate::nfa::Graph {
            states: self
                .states
                .iter()
                .map(|state| crate::nfa::State {
                    epsilon: std::collections::BTreeSet::new(),
                    non_epsilon: state
                        .transitions
                        .iter()
                        .map(|(token, &(dst, fn_name))| {
                            (token.clone(), (core::iter::once(dst).collect(), fn_name))
                        })
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

    /// Find the minimal input that reaches this state.
    /// Like Dijkstra's but optimized to leverage that each edge is 1 unit long
    #[inline]
    #[must_use]
    #[allow(clippy::panic_in_result_fn, clippy::unwrap_in_result)]
    pub(crate) fn dijkstra(&self, initial: usize, endpoint: usize) -> Vec<I> {
        use core::cmp::Reverse;
        use std::collections::BinaryHeap;

        let mut cache = BTreeMap::<usize, Vec<I>>::new();
        let mut queue = BinaryHeap::new();

        drop(cache.insert(initial, vec![]));
        queue.push(Reverse(CmpFirst(0_usize, initial)));

        while let Some(Reverse(CmpFirst(distance, index))) = queue.pop() {
            let cached = unwrap!(cache.get(&index)).clone(); // TODO: look into `Cow`
            if index == endpoint {
                return cached;
            }
            let state = get!(self.states, index);
            for (token, &(next, _fn_name)) in &state.transitions {
                if let Entry::Vacant(entry) = cache.entry(next) {
                    entry.insert(cached.clone()).push(token.clone());
                    queue.push(Reverse(CmpFirst(distance.saturating_add(1), next)));
                }
            }
        }

        #[allow(clippy::unreachable)]
        #[cfg(any(test, debug_assertions))]
        {
            unreachable!()
        }

        #[allow(unsafe_code)]
        #[cfg(not(any(test, debug_assertions)))]
        unsafe {
            core::hint::unreachable_unchecked()
        }
    }

    /// Find the minimal input that reaches this state.
    #[inline]
    #[must_use]
    pub(crate) fn backtrack(&self, endpoint: usize) -> Vec<I> {
        self.dijkstra(self.initial, endpoint)
    }

    /// Print as a set of Rust source-code functions.
    #[inline]
    #[must_use]
    #[allow(clippy::too_many_lines, unsafe_code)]
    pub fn into_source(self, name: &str) -> String
    where
        I: Expression,
    {
        let (ast_fn, ast_mod) = self.to_ast(name);
        let (rev_fn, rev_mod) = self
            .generalize()
            .reverse()
            .compile()
            .to_fuzz_ast(&format!("{name}_fuzz"));
        prettyplease::unparse(&syn::File {
            shebang: None,
            attrs: vec![],
            items: vec![
                syn::Item::Fn(ast_fn),
                syn::Item::Mod(ast_mod),
                syn::Item::Fn(rev_fn),
                syn::Item::Mod(rev_mod),
            ],
        })
    }

    /// Print as a set of Rust source-code functions.
    #[inline]
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn to_ast(&self, name: &str) -> (syn::ItemFn, syn::ItemMod)
    where
        I: Expression,
    {
        let generics = syn::Generics {
            lt_token: Some(Token!(<)(Span::call_site())),
            params: core::iter::once(syn::GenericParam::Type(syn::TypeParam {
                attrs: vec![],
                ident: Ident::new("I", Span::call_site()),
                colon_token: Some(Token!(:)(Span::call_site())),
                bounds: core::iter::once(syn::TypeParamBound::Trait(syn::TraitBound {
                    paren_token: None,
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
                                            ty: I::to_type(),
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
            }))
            .collect(),
            gt_token: Some(Token!(>)(Span::call_site())),
            where_clause: None,
        };
        let states = syn::ItemMod {
            attrs: vec![
                syn::Attribute {
                    pound_token: Token!(#)(Span::call_site()),
                    style: syn::AttrStyle::Outer,
                    bracket_token: syn::token::Bracket::default(),
                    meta: syn::Meta::List(syn::MetaList {
                        path: syn::Path {
                            leading_colon: None,
                            segments: core::iter::once(syn::PathSegment {
                                ident: Ident::new("allow", Span::call_site()),
                                arguments: syn::PathArguments::None,
                            })
                            .collect(),
                        },
                        delimiter: syn::MacroDelimiter::Paren(syn::token::Paren::default()),
                        tokens: Ident::new("non_snake_case", Span::call_site()).into_token_stream(),
                    }),
                },
                syn::Attribute {
                    pound_token: Token!(#)(Span::call_site()),
                    style: syn::AttrStyle::Outer,
                    bracket_token: syn::token::Bracket::default(),
                    meta: syn::Meta::List(syn::MetaList {
                        path: syn::Path {
                            leading_colon: None,
                            segments: core::iter::once(syn::PathSegment {
                                ident: Ident::new("allow", Span::call_site()),
                                arguments: syn::PathArguments::None,
                            })
                            .collect(),
                        },
                        delimiter: syn::MacroDelimiter::Paren(syn::token::Paren::default()),
                        tokens: Ident::new("non_camel_case_types", Span::call_site())
                            .into_token_stream(),
                    }),
                },
                syn::Attribute {
                    pound_token: Token!(#)(Span::call_site()),
                    style: syn::AttrStyle::Outer,
                    bracket_token: syn::token::Bracket::default(),
                    meta: syn::Meta::List(syn::MetaList {
                        path: syn::Path {
                            leading_colon: None,
                            segments: core::iter::once(syn::PathSegment {
                                ident: Ident::new("allow", Span::call_site()),
                                arguments: syn::PathArguments::None,
                            })
                            .collect(),
                        },
                        delimiter: syn::MacroDelimiter::Paren(syn::token::Paren::default()),
                        tokens: Ident::new("unused_parens", Span::call_site()).into_token_stream(),
                    }),
                },
                syn::Attribute {
                    pound_token: Token!(#)(Span::call_site()),
                    style: syn::AttrStyle::Outer,
                    bracket_token: syn::token::Bracket::default(),
                    meta: syn::Meta::List(syn::MetaList {
                        path: syn::Path {
                            leading_colon: None,
                            segments: core::iter::once(syn::PathSegment {
                                ident: Ident::new("allow", Span::call_site()),
                                arguments: syn::PathArguments::None,
                            })
                            .collect(),
                        },
                        delimiter: syn::MacroDelimiter::Paren(syn::token::Paren::default()),
                        tokens: Ident::new("unused_variables", Span::call_site())
                            .into_token_stream(),
                    }),
                },
            ],
            vis: syn::Visibility::Restricted(syn::VisRestricted {
                pub_token: Token!(pub)(Span::call_site()),
                paren_token: syn::token::Paren::default(),
                in_token: None,
                path: Box::new(syn::Path {
                    leading_colon: None,
                    segments: core::iter::once(syn::PathSegment {
                        ident: Ident::new("crate", Span::call_site()),
                        arguments: syn::PathArguments::None,
                    })
                    .collect(),
                }),
            }),
            unsafety: None,
            mod_token: Token!(mod)(Span::call_site()),
            ident: Ident::new(&format!("{name}_states"), Span::call_site()),
            content: Some((
                syn::token::Brace::default(),
                self.states
                    .iter()
                    .enumerate()
                    .flat_map(|(i, state)| {
                        state.to_source(i, name, generics.clone(), &self.backtrack(i))
                    })
                    .collect(),
            )),
            semi: None,
        };
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
                                                &format!("{name}_states"),
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
                            args: [
                                syn::Expr::Path(syn::ExprPath {
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
                                }),
                                syn::Expr::Call(syn::ExprCall {
                                    attrs: vec![],
                                    func: Box::new(syn::Expr::Path(syn::ExprPath {
                                        attrs: vec![],
                                        qself: None,
                                        path: config_path(name, "initial"),
                                    })),
                                    paren_token: syn::token::Paren::default(),
                                    args: syn::punctuated::Punctuated::new(),
                                }),
                            ]
                            .into_iter()
                            .collect(),
                        }),
                        None,
                    )],
                }),
                sig: syn::Signature {
                    constness: None,
                    asyncness: None,
                    unsafety: None,
                    abi: None,
                    fn_token: Token!(fn)(Span::call_site()),
                    ident: Ident::new(name, Span::call_site()),
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
                                    ident: Ident::new("Option", Span::call_site()),
                                    arguments: syn::PathArguments::AngleBracketed(
                                        syn::AngleBracketedGenericArguments {
                                            colon2_token: None,
                                            lt_token: Token!(<)(Span::call_site()),
                                            args: core::iter::once(syn::GenericArgument::Type(
                                                output_type(name),
                                            ))
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
                vis: syn::Visibility::Public(Token!(pub)(Span::call_site())),
            },
            states,
        )
    }

    /// Print as a set of Rust source-code functions.
    #[inline]
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn to_fuzz_ast(&self, name: &str) -> (syn::ItemFn, syn::ItemMod)
    where
        I: Expression,
    {
        let generics = syn::Generics {
            lt_token: Some(Token!(<)(Span::call_site())),
            params: core::iter::once(syn::GenericParam::Type(syn::TypeParam {
                attrs: vec![],
                ident: Ident::new("R", Span::call_site()),
                colon_token: Some(Token!(:)(Span::call_site())),
                bounds: core::iter::once(syn::TypeParamBound::Trait(syn::TraitBound {
                    paren_token: None,
                    modifier: syn::TraitBoundModifier::None,
                    lifetimes: None,
                    path: syn::Path {
                        leading_colon: None,
                        segments: [
                            syn::PathSegment {
                                ident: Ident::new("rand", Span::call_site()),
                                arguments: syn::PathArguments::None,
                            },
                            syn::PathSegment {
                                ident: Ident::new("Rng", Span::call_site()),
                                arguments: syn::PathArguments::None,
                            },
                        ]
                        .into_iter()
                        .collect(),
                    },
                }))
                .collect(),
                eq_token: None,
                default: None,
            }))
            .collect(),
            gt_token: Some(Token!(>)(Span::call_site())),
            where_clause: None,
        };
        let states = syn::ItemMod {
            attrs: vec![
                syn::Attribute {
                    pound_token: Token!(#)(Span::call_site()),
                    style: syn::AttrStyle::Outer,
                    bracket_token: syn::token::Bracket::default(),
                    meta: syn::Meta::List(syn::MetaList {
                        path: syn::Path {
                            leading_colon: None,
                            segments: core::iter::once(syn::PathSegment {
                                ident: Ident::new("allow", Span::call_site()),
                                arguments: syn::PathArguments::None,
                            })
                            .collect(),
                        },
                        delimiter: syn::MacroDelimiter::Paren(syn::token::Paren::default()),
                        tokens: Ident::new("non_snake_case", Span::call_site()).into_token_stream(),
                    }),
                },
                syn::Attribute {
                    pound_token: Token!(#)(Span::call_site()),
                    style: syn::AttrStyle::Outer,
                    bracket_token: syn::token::Bracket::default(),
                    meta: syn::Meta::List(syn::MetaList {
                        path: syn::Path {
                            leading_colon: None,
                            segments: core::iter::once(syn::PathSegment {
                                ident: Ident::new("allow", Span::call_site()),
                                arguments: syn::PathArguments::None,
                            })
                            .collect(),
                        },
                        delimiter: syn::MacroDelimiter::Paren(syn::token::Paren::default()),
                        tokens: Ident::new("unused_mut", Span::call_site()).into_token_stream(),
                    }),
                },
            ],
            vis: syn::Visibility::Inherited,
            unsafety: None,
            mod_token: Token!(mod)(Span::call_site()),
            ident: Ident::new(&format!("{name}_states"), Span::call_site()),
            content: Some((
                syn::token::Brace::default(),
                self.states
                    .iter()
                    .enumerate()
                    .map(|(i, state)| {
                        state.to_fuzz_source(i, self.initial, generics.clone(), &self.backtrack(i))
                    })
                    .collect(),
            )),
            semi: None,
        };
        (
            syn::ItemFn {
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
                    ident: Ident::new(name, Span::call_site()),
                    generics,
                    paren_token: syn::token::Paren::default(),
                    inputs: core::iter::once(syn::FnArg::Typed(syn::PatType {
                        attrs: vec![],
                        pat: Box::new(syn::Pat::Ident(syn::PatIdent {
                            attrs: vec![],
                            by_ref: None,
                            mutability: None,
                            ident: Ident::new("r", Span::call_site()),
                            subpat: None,
                        })),
                        colon_token: Token!(:)(Span::call_site()),
                        ty: Box::new(syn::Type::Reference(syn::TypeReference {
                            and_token: Token!(&)(Span::call_site()),
                            lifetime: None,
                            mutability: Some(Token!(mut)(Span::call_site())),
                            elem: Box::new(syn::Type::Path(syn::TypePath {
                                qself: None,
                                path: syn::Path {
                                    leading_colon: None,
                                    segments: core::iter::once(syn::PathSegment {
                                        ident: Ident::new("R", Span::call_site()),
                                        arguments: syn::PathArguments::None,
                                    })
                                    .collect(),
                                },
                            })),
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
                                    ident: Ident::new("Vec", Span::call_site()),
                                    arguments: syn::PathArguments::AngleBracketed(
                                        syn::AngleBracketedGenericArguments {
                                            colon2_token: None,
                                            lt_token: Token!(<)(Span::call_site()),
                                            args: core::iter::once(syn::GenericArgument::Type(
                                                I::to_type(),
                                            ))
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
                    stmts: vec![
                        syn::Stmt::Expr(
                            syn::Expr::Let(syn::ExprLet {
                                attrs: vec![],
                                let_token: Token!(let)(Span::call_site()),
                                pat: Box::new(syn::Pat::Ident(syn::PatIdent {
                                    attrs: vec![],
                                    by_ref: None,
                                    mutability: Some(Token!(mut)(Span::call_site())),
                                    ident: Ident::new("v", Span::call_site()),
                                    subpat: None,
                                })),
                                eq_token: Token!(=)(Span::call_site()),
                                expr: Box::new(syn::Expr::Call(syn::ExprCall {
                                    attrs: vec![],
                                    func: Box::new(syn::Expr::Path(syn::ExprPath {
                                        attrs: vec![],
                                        qself: None,
                                        path: syn::Path {
                                            leading_colon: None,
                                            segments: [
                                                syn::PathSegment {
                                                    ident: Ident::new(
                                                        &format!("{name}_states"),
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
                                    args: [
                                        syn::Expr::Path(syn::ExprPath {
                                            attrs: vec![],
                                            qself: None,
                                            path: syn::Path {
                                                leading_colon: None,
                                                segments: core::iter::once(syn::PathSegment {
                                                    ident: Ident::new("r", Span::call_site()),
                                                    arguments: syn::PathArguments::None,
                                                })
                                                .collect(),
                                            },
                                        }),
                                        syn::Expr::Macro(syn::ExprMacro {
                                            attrs: vec![],
                                            mac: syn::Macro {
                                                path: syn::Path {
                                                    leading_colon: None,
                                                    segments: core::iter::once(syn::PathSegment {
                                                        ident: Ident::new("vec", Span::call_site()),
                                                        arguments: syn::PathArguments::None,
                                                    })
                                                    .collect(),
                                                },
                                                bang_token: Token!(!)(Span::call_site()),
                                                delimiter: syn::MacroDelimiter::Bracket(
                                                    syn::token::Bracket::default(),
                                                ),
                                                tokens: proc_macro2::TokenStream::new(),
                                            },
                                        }),
                                    ]
                                    .into_iter()
                                    .collect(),
                                })),
                            }),
                            Some(Token!(;)(Span::call_site())),
                        ),
                        syn::Stmt::Expr(
                            syn::Expr::MethodCall(syn::ExprMethodCall {
                                attrs: vec![],
                                receiver: Box::new(syn::Expr::Path(syn::ExprPath {
                                    attrs: vec![],
                                    qself: None,
                                    path: syn::Path {
                                        leading_colon: None,
                                        segments: core::iter::once(syn::PathSegment {
                                            ident: Ident::new("v", Span::call_site()),
                                            arguments: syn::PathArguments::None,
                                        })
                                        .collect(),
                                    },
                                })),
                                dot_token: Token!(.)(Span::call_site()),
                                method: Ident::new("reverse", Span::call_site()),
                                turbofish: None,
                                paren_token: syn::token::Paren::default(),
                                args: syn::punctuated::Punctuated::new(),
                            }),
                            Some(Token!(;)(Span::call_site())),
                        ),
                        syn::Stmt::Expr(
                            syn::Expr::Path(syn::ExprPath {
                                attrs: vec![],
                                qself: None,
                                path: syn::Path {
                                    leading_colon: None,
                                    segments: core::iter::once(syn::PathSegment {
                                        ident: Ident::new("v", Span::call_site()),
                                        arguments: syn::PathArguments::None,
                                    })
                                    .collect(),
                                },
                            }),
                            None,
                        ),
                    ],
                }),
            },
            states,
        )
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

impl<I: Clone + Ord + Expression> core::fmt::Display for Graph<I> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Initial state: {}", self.initial)?;
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
        for (input, &(transitions, fn_name)) in &self.transitions {
            writeln!(f, "    {input:?} --> {transitions} >>= {fn_name:?}")?;
        }
        Ok(())
    }
}

/// User-defined config module after build time.
#[inline]
fn config_path(name: &str, destination: &str) -> syn::Path {
    syn::Path {
        leading_colon: None,
        segments: [
            syn::PathSegment {
                ident: Ident::new("crate", Span::call_site()),
                arguments: syn::PathArguments::None,
            },
            syn::PathSegment {
                ident: Ident::new("inator_config", Span::call_site()),
                arguments: syn::PathArguments::None,
            },
            syn::PathSegment {
                ident: Ident::new(name, Span::call_site()),
                arguments: syn::PathArguments::None,
            },
            syn::PathSegment {
                ident: Ident::new(destination, Span::call_site()),
                arguments: syn::PathArguments::None,
            },
        ]
        .into_iter()
        .collect(),
    }
}

/// Reference the user-defined output type after build time.
#[inline]
fn output_type(name: &str) -> syn::Type {
    syn::Type::Path(syn::TypePath {
        qself: None,
        path: config_path(name, "Output"),
    })
}

/// Map from outputs to all inputs which were mapped to them.
#[inline]
fn invert<K: Ord, V: Ord>(map: &BTreeMap<K, V>) -> BTreeMap<&V, BTreeSet<&K>> {
    let mut acc = BTreeMap::<&V, BTreeSet<&K>>::new();
    for (k, v) in map {
        let _ = acc.entry(v).or_default().insert(k);
    }
    acc
}

impl<I: Clone + Ord> State<I> {
    /// State to which this state can transition on a given input.
    #[inline]
    pub fn transition(&self, input: &I) -> Option<&(usize, Option<&'static str>)> {
        self.transitions.get(input)
    }

    /// Print as a Rust source-code function.
    #[inline]
    #[allow(clippy::too_many_lines)]
    pub fn to_source(
        &self,
        index: usize,
        name: &str,
        generics: syn::Generics,
        minimal_input: &[I],
    ) -> Vec<syn::Item>
    where
        I: Expression,
    {
        let inverted = invert(&self.transitions);
        vec![syn::Item::Fn(syn::ItemFn {
            attrs: vec![
                syn::Attribute {
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
                },
                syn::Attribute {
                    pound_token: Token!(#)(Span::call_site()),
                    style: syn::AttrStyle::Outer,
                    bracket_token: syn::token::Bracket::default(),
                    meta: syn::Meta::NameValue(syn::MetaNameValue {
                        path: syn::Path {
                            leading_colon: None,
                            segments: core::iter::once(syn::PathSegment {
                                ident: Ident::new("doc", Span::call_site()),
                                arguments: syn::PathArguments::None,
                            })
                            .collect(),
                        },
                        eq_token: Token!(=)(Span::call_site()),
                        value: syn::Expr::Lit(syn::ExprLit {
                            attrs: vec![],
                            lit: syn::Lit::Str(syn::LitStr::new(
                                &format!(
                                    "Minimal input to reach this state: {}",
                                    match minimal_input.split_first() {
                                        None => "[this is the initial state]".to_owned(),
                                        Some((head, tail)) =>
                                            tail.iter().fold(format!("{head:?}"), |acc, token| {
                                                acc + &format!(" -> {token:?}")
                                            }),
                                    },
                                ),
                                Span::call_site(),
                            )),
                        }),
                    }),
                },
            ],
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
                inputs: [
                    syn::FnArg::Typed(syn::PatType {
                        attrs: vec![],
                        pat: Box::new(syn::Pat::Ident(syn::PatIdent {
                            attrs: vec![],
                            by_ref: None,
                            mutability: Some(Token!(mut)(Span::call_site())),
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
                    }),
                    syn::FnArg::Typed(syn::PatType {
                        attrs: vec![],
                        pat: Box::new(syn::Pat::Ident(syn::PatIdent {
                            attrs: vec![],
                            by_ref: None,
                            mutability: None,
                            ident: Ident::new("acc", Span::call_site()),
                            subpat: None,
                        })),
                        colon_token: Token!(:)(Span::call_site()),
                        ty: Box::new(output_type(name)),
                    }),
                ]
                .into_iter()
                .collect(),
                variadic: None,
                output: syn::ReturnType::Type(
                    Token!(->)(Span::call_site()),
                    Box::new(syn::Type::Path(syn::TypePath {
                        qself: None,
                        path: syn::Path {
                            leading_colon: None,
                            segments: core::iter::once(syn::PathSegment {
                                ident: Ident::new("Option", Span::call_site()),
                                arguments: syn::PathArguments::AngleBracketed(
                                    syn::AngleBracketedGenericArguments {
                                        colon2_token: None,
                                        lt_token: Token!(<)(Span::call_site()),
                                        args: core::iter::once(syn::GenericArgument::Type(
                                            output_type(name),
                                        ))
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
                        arms: {
                            let mut v = vec![];
                            for (&&(dst, fn_name), inputs) in &inverted {
                                let sdst = format!("s{dst}");
                                v.push(syn::Arm {
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
                                        elems: core::iter::once(syn::Pat::Ident(syn::PatIdent{
                                            attrs: vec![],
                                            by_ref: None,
                                            mutability: None,
                                            ident: Ident::new("token", Span::call_site()),
                                            subpat: Some((
                                                Token!(@)(Span::call_site()),
                                                Box::new(syn::Pat::Paren(syn::PatParen {
                                                    attrs: vec![],
                                                    paren_token: syn::token::Paren::default(),
                                                    pat: Box::new(syn::Pat::Or(syn::PatOr {
                                                        attrs: vec![],
                                                        leading_vert: None,
                                                        cases: inputs.iter().map(|input| input.to_pattern()).collect(),
                                                    })),
                                                })),
                                            )),
                                        })).collect(),
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
                                                    ident: Ident::new(&sdst, Span::call_site()),
                                                    arguments: syn::PathArguments::None,
                                                })
                                                .collect(),
                                            },
                                        })),
                                        paren_token: syn::token::Paren::default(),
                                        args: [
                                            syn::Expr::Path(syn::ExprPath {
                                                attrs: vec![],
                                                qself: None,
                                                path: syn::Path {
                                                    leading_colon: None,
                                                    segments: core::iter::once(
                                                        syn::PathSegment {
                                                            ident: Ident::new(
                                                                "i",
                                                                Span::call_site(),
                                                            ),
                                                            arguments: syn::PathArguments::None,
                                                        },
                                                    )
                                                    .collect(),
                                                },
                                            }),
                                            fn_name.map_or_else(|| {
                                                syn::Expr::Path(syn::ExprPath {
                                                    attrs: vec![],
                                                    qself: None,
                                                    path: syn::Path {
                                                        leading_colon: None,
                                                        segments: core::iter::once(
                                                            syn::PathSegment {
                                                                ident: Ident::new(
                                                                    "acc",
                                                                    Span::call_site(),
                                                                ),
                                                                arguments:
                                                                    syn::PathArguments::None,
                                                            },
                                                        )
                                                        .collect(),
                                                    },
                                                })
                                            }, |f| {
                                                syn::Expr::Call(syn::ExprCall {
                                                    attrs: vec![],
                                                    func: Box::new(syn::Expr::Path(
                                                        syn::ExprPath {
                                                            attrs: vec![],
                                                            qself: None,
                                                            path: config_path(name, f),
                                                        },
                                                    )),
                                                    paren_token: syn::token::Paren::default(),
                                                    args: [
                                                        syn::Expr::Path(syn::ExprPath {
                                                            attrs: vec![],
                                                            qself: None,
                                                            path: syn::Path {
                                                                leading_colon: None,
                                                                segments: core::iter::once(
                                                                    syn::PathSegment {
                                                                        ident: Ident::new(
                                                                            "acc",
                                                                            Span::call_site(),
                                                                        ),
                                                                        arguments:
                                                                            syn::PathArguments::None,
                                                                    },
                                                                )
                                                                .collect(),
                                                            },
                                                        }),
                                                        syn::Expr::Path(syn::ExprPath {
                                                            attrs: vec![],
                                                            qself: None,
                                                            path: syn::Path {
                                                                leading_colon: None,
                                                                segments: core::iter::once(
                                                                    syn::PathSegment {
                                                                        ident: Ident::new(
                                                                            "token",
                                                                            Span::call_site(),
                                                                        ),
                                                                        arguments:
                                                                            syn::PathArguments::None,
                                                                    },
                                                                )
                                                                .collect(),
                                                            },
                                                        }),
                                                    ]
                                                    .into_iter()
                                                    .collect(),
                                                })
                                            }),
                                        ]
                                        .into_iter()
                                        .collect(),
                                    })),
                                    comma: Some(Token!(,)(Span::call_site())),
                                });
                            }
                            v.push(syn::Arm {
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
                                                    ident: Ident::new("Some", Span::call_site()),
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
                                                    ident: Ident::new("acc", Span::call_site()),
                                                    arguments: syn::PathArguments::None,
                                                })
                                                .collect(),
                                            },
                                        }))
                                        .collect(),
                                    })
                                } else {
                                    syn::Expr::Path(syn::ExprPath {
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
                                    })
                                }),
                                comma: Some(Token!(,)(Span::call_site())),
                            });
                            v.push(syn::Arm {
                                attrs: vec![],
                                pat: syn::Pat::Wild(syn::PatWild {
                                    attrs: vec![],
                                    underscore_token: Token!(_)(Span::call_site()),
                                }),
                                guard: None,
                                fat_arrow_token: Token!(=>)(Span::call_site()),
                                body: Box::new(syn::Expr::Path(syn::ExprPath {
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
                                })),
                                comma: Some(Token!(,)(Span::call_site())),
                            });
                            v
                        },
                    }),
                    None,
                )],
            }),
        })]
    }

    /// Print as a Rust source-code function.
    #[inline]
    #[allow(clippy::too_many_lines)]
    pub fn to_fuzz_source(
        &self,
        index: usize,
        initial: usize,
        generics: syn::Generics,
        minimal_input: &[I],
    ) -> syn::Item
    where
        I: Expression,
    {
        let mut stmts = vec![];
        if self.accepting {
            stmts.push(syn::Stmt::Expr(
                syn::Expr::If(syn::ExprIf {
                    attrs: vec![],
                    if_token: Token!(if)(Span::call_site()),
                    cond: Box::new(syn::Expr::MethodCall(syn::ExprMethodCall {
                        attrs: vec![],
                        receiver: Box::new(syn::Expr::Path(syn::ExprPath {
                            attrs: vec![],
                            qself: None,
                            path: syn::Path {
                                leading_colon: None,
                                segments: core::iter::once(syn::PathSegment {
                                    ident: Ident::new("r", Span::call_site()),
                                    arguments: syn::PathArguments::None,
                                })
                                .collect(),
                            },
                        })),
                        dot_token: Token!(.)(Span::call_site()),
                        method: Ident::new("gen", Span::call_site()),
                        turbofish: None,
                        paren_token: syn::token::Paren::default(),
                        args: syn::punctuated::Punctuated::new(),
                    })),
                    then_branch: syn::Block {
                        brace_token: syn::token::Brace::default(),
                        stmts: vec![syn::Stmt::Expr(
                            syn::Expr::Return(syn::ExprReturn {
                                attrs: vec![],
                                return_token: Token!(return)(Span::call_site()),
                                expr: Some(Box::new(syn::Expr::Path(syn::ExprPath {
                                    attrs: vec![],
                                    qself: None,
                                    path: syn::Path {
                                        leading_colon: None,
                                        segments: core::iter::once(syn::PathSegment {
                                            ident: Ident::new("acc", Span::call_site()),
                                            arguments: syn::PathArguments::None,
                                        })
                                        .collect(),
                                    },
                                }))),
                            }),
                            Some(Token!(;)(Span::call_site())),
                        )],
                    },
                    else_branch: None,
                }),
                None,
            ));
        }
        match self.transitions.len() {
            0 => stmts.push(syn::Stmt::Expr(
                syn::Expr::Call(syn::ExprCall {
                    attrs: vec![],
                    func: Box::new(syn::Expr::Path(syn::ExprPath {
                        attrs: vec![],
                        qself: None,
                        path: syn::Path {
                            leading_colon: None,
                            segments: core::iter::once(syn::PathSegment {
                                ident: Ident::new(&format!("s{initial}"), Span::call_site()),
                                arguments: syn::PathArguments::None,
                            })
                            .collect(),
                        },
                    })),
                    paren_token: syn::token::Paren::default(),
                    args: [
                        syn::Expr::Path(syn::ExprPath {
                            attrs: vec![],
                            qself: None,
                            path: syn::Path {
                                leading_colon: None,
                                segments: core::iter::once(syn::PathSegment {
                                    ident: Ident::new("r", Span::call_site()),
                                    arguments: syn::PathArguments::None,
                                })
                                .collect(),
                            },
                        }),
                        syn::Expr::Macro(syn::ExprMacro {
                            attrs: vec![],
                            mac: syn::Macro {
                                path: syn::Path {
                                    leading_colon: None,
                                    segments: core::iter::once(syn::PathSegment {
                                        ident: Ident::new("vec", Span::call_site()),
                                        arguments: syn::PathArguments::None,
                                    })
                                    .collect(),
                                },
                                bang_token: Token!(!)(Span::call_site()),
                                delimiter: syn::MacroDelimiter::Bracket(
                                    syn::token::Bracket::default(),
                                ),
                                tokens: proc_macro2::TokenStream::new(),
                            },
                        }),
                    ]
                    .into_iter()
                    .collect(),
                }),
                None,
            )),
            1 => stmts.extend({
                let (token, &(dst, _fn_name)) = unwrap!(self.transitions.first_key_value());
                fuzz_stmts(token, dst)
            }),
            _ => stmts.push(syn::Stmt::Expr(
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
                                    ident: Ident::new("r", Span::call_site()),
                                    arguments: syn::PathArguments::None,
                                })
                                .collect(),
                            },
                        })),
                        dot_token: Token!(.)(Span::call_site()),
                        method: Ident::new("gen_range", Span::call_site()),
                        turbofish: None,
                        paren_token: syn::token::Paren::default(),
                        args: core::iter::once(syn::Expr::Range(syn::ExprRange {
                            attrs: vec![],
                            start: Some(Box::new(syn::Expr::Lit(syn::ExprLit {
                                attrs: vec![],
                                lit: syn::Lit::Int(syn::LitInt::new("0", Span::call_site())),
                            }))),
                            limits: syn::RangeLimits::HalfOpen(Token!(..)(Span::call_site())),
                            end: Some(Box::new(syn::Expr::Lit(syn::ExprLit {
                                attrs: vec![],
                                lit: syn::Lit::Int(syn::LitInt::new(
                                    &format!("{}", self.transitions.len()),
                                    Span::call_site(),
                                )),
                            }))),
                        }))
                        .collect(),
                    })),
                    brace_token: syn::token::Brace::default(),
                    arms: self
                        .transitions
                        .iter()
                        .enumerate()
                        .map(|(i, (token, &(dst, _fn_name)))| syn::Arm {
                            attrs: vec![],
                            pat: syn::Pat::Lit(syn::ExprLit {
                                attrs: vec![],
                                lit: syn::Lit::Int(syn::LitInt::new(
                                    &format!("{i}"),
                                    Span::call_site(),
                                )),
                            }),
                            guard: None,
                            fat_arrow_token: Token!(=>)(Span::call_site()),
                            body: Box::new(syn::Expr::Block(syn::ExprBlock {
                                attrs: vec![],
                                label: None,
                                block: syn::Block {
                                    brace_token: syn::token::Brace::default(),
                                    stmts: fuzz_stmts(token, dst),
                                },
                            })),
                            comma: None,
                        })
                        .chain(core::iter::once(syn::Arm {
                            attrs: vec![],
                            pat: syn::Pat::Wild(syn::PatWild {
                                attrs: vec![],
                                underscore_token: Token!(_)(Span::call_site()),
                            }),
                            guard: None,
                            fat_arrow_token: Token!(=>)(Span::call_site()),
                            body: Box::new(syn::Expr::Unsafe(syn::ExprUnsafe {
                                attrs: vec![],
                                unsafe_token: Token!(unsafe)(Span::call_site()),
                                block: syn::Block {
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
                                                                "core",
                                                                Span::call_site(),
                                                            ),
                                                            arguments: syn::PathArguments::None,
                                                        },
                                                        syn::PathSegment {
                                                            ident: Ident::new(
                                                                "hint",
                                                                Span::call_site(),
                                                            ),
                                                            arguments: syn::PathArguments::None,
                                                        },
                                                        syn::PathSegment {
                                                            ident: Ident::new(
                                                                "unreachable_unchecked",
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
                                            args: syn::punctuated::Punctuated::new(),
                                        }),
                                        None,
                                    )],
                                },
                            })),
                            comma: None,
                        }))
                        .collect(),
                }),
                None,
            )),
        }
        syn::Item::Fn(syn::ItemFn {
            attrs: vec![
                syn::Attribute {
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
                },
                syn::Attribute {
                    pound_token: Token!(#)(Span::call_site()),
                    style: syn::AttrStyle::Outer,
                    bracket_token: syn::token::Bracket::default(),
                    meta: syn::Meta::NameValue(syn::MetaNameValue {
                        path: syn::Path {
                            leading_colon: None,
                            segments: core::iter::once(syn::PathSegment {
                                ident: Ident::new("doc", Span::call_site()),
                                arguments: syn::PathArguments::None,
                            })
                            .collect(),
                        },
                        eq_token: Token!(=)(Span::call_site()),
                        value: syn::Expr::Lit(syn::ExprLit {
                            attrs: vec![],
                            lit: syn::Lit::Str(syn::LitStr::new(
                                &format!(
                                    "Minimal input to reach this state: {}",
                                    match minimal_input.split_first() {
                                        None => "[this is the initial state]".to_owned(),
                                        Some((head, tail)) =>
                                            tail.iter().fold(format!("{head:?}"), |acc, token| {
                                                acc + &format!(" -> {token:?}")
                                            }),
                                    },
                                ),
                                Span::call_site(),
                            )),
                        }),
                    }),
                },
            ],
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
                inputs: [
                    syn::FnArg::Typed(syn::PatType {
                        attrs: vec![],
                        pat: Box::new(syn::Pat::Ident(syn::PatIdent {
                            attrs: vec![],
                            by_ref: None,
                            mutability: None,
                            ident: Ident::new("r", Span::call_site()),
                            subpat: None,
                        })),
                        colon_token: Token!(:)(Span::call_site()),
                        ty: Box::new(syn::Type::Reference(syn::TypeReference {
                            and_token: Token!(&)(Span::call_site()),
                            lifetime: None,
                            mutability: Some(Token!(mut)(Span::call_site())),
                            elem: Box::new(syn::Type::Path(syn::TypePath {
                                qself: None,
                                path: syn::Path {
                                    leading_colon: None,
                                    segments: core::iter::once(syn::PathSegment {
                                        ident: Ident::new("R", Span::call_site()),
                                        arguments: syn::PathArguments::None,
                                    })
                                    .collect(),
                                },
                            })),
                        })),
                    }),
                    syn::FnArg::Typed(syn::PatType {
                        attrs: vec![],
                        pat: Box::new(syn::Pat::Ident(syn::PatIdent {
                            attrs: vec![],
                            by_ref: None,
                            mutability: Some(Token!(mut)(Span::call_site())),
                            ident: Ident::new("acc", Span::call_site()),
                            subpat: None,
                        })),
                        colon_token: Token!(:)(Span::call_site()),
                        ty: Box::new(syn::Type::Path(syn::TypePath {
                            qself: None,
                            path: syn::Path {
                                leading_colon: None,
                                segments: core::iter::once(syn::PathSegment {
                                    ident: Ident::new("Vec", Span::call_site()),
                                    arguments: syn::PathArguments::AngleBracketed(
                                        syn::AngleBracketedGenericArguments {
                                            colon2_token: None,
                                            lt_token: Token!(<)(Span::call_site()),
                                            args: core::iter::once(syn::GenericArgument::Type(
                                                I::to_type(),
                                            ))
                                            .collect(),
                                            gt_token: Token!(>)(Span::call_site()),
                                        },
                                    ),
                                })
                                .collect(),
                            },
                        })),
                    }),
                ]
                .into_iter()
                .collect(),
                variadic: None,
                output: syn::ReturnType::Type(
                    Token!(->)(Span::call_site()),
                    Box::new(syn::Type::Path(syn::TypePath {
                        qself: None,
                        path: syn::Path {
                            leading_colon: None,
                            segments: core::iter::once(syn::PathSegment {
                                ident: Ident::new("Vec", Span::call_site()),
                                arguments: syn::PathArguments::AngleBracketed(
                                    syn::AngleBracketedGenericArguments {
                                        colon2_token: None,
                                        lt_token: Token!(<)(Span::call_site()),
                                        args: core::iter::once(syn::GenericArgument::Type(
                                            I::to_type(),
                                        ))
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
                stmts,
            }),
        })
    }
}

/// When writing a fuzzer function, append this character then jump to this other state.
fn fuzz_stmts<E: Expression>(token: &E, dst: usize) -> Vec<syn::Stmt> {
    vec![
        syn::Stmt::Expr(
            syn::Expr::MethodCall(syn::ExprMethodCall {
                attrs: vec![],
                receiver: Box::new(syn::Expr::Path(syn::ExprPath {
                    attrs: vec![],
                    qself: None,
                    path: syn::Path {
                        leading_colon: None,
                        segments: core::iter::once(syn::PathSegment {
                            ident: Ident::new("acc", Span::call_site()),
                            arguments: syn::PathArguments::None,
                        })
                        .collect(),
                    },
                })),
                dot_token: Token!(.)(Span::call_site()),
                method: Ident::new("push", Span::call_site()),
                turbofish: None,
                paren_token: syn::token::Paren::default(),
                args: core::iter::once(token.to_expr()).collect(),
            }),
            Some(Token!(;)(Span::call_site())),
        ),
        syn::Stmt::Expr(
            syn::Expr::Call(syn::ExprCall {
                attrs: vec![],
                func: Box::new(syn::Expr::Path(syn::ExprPath {
                    attrs: vec![],
                    qself: None,
                    path: syn::Path {
                        leading_colon: None,
                        segments: core::iter::once(syn::PathSegment {
                            ident: Ident::new(&format!("s{dst}"), Span::call_site()),
                            arguments: syn::PathArguments::None,
                        })
                        .collect(),
                    },
                })),
                paren_token: syn::token::Paren::default(),
                args: [
                    syn::Expr::Path(syn::ExprPath {
                        attrs: vec![],
                        qself: None,
                        path: syn::Path {
                            leading_colon: None,
                            segments: core::iter::once(syn::PathSegment {
                                ident: Ident::new("r", Span::call_site()),
                                arguments: syn::PathArguments::None,
                            })
                            .collect(),
                        },
                    }),
                    syn::Expr::Path(syn::ExprPath {
                        attrs: vec![],
                        qself: None,
                        path: syn::Path {
                            leading_colon: None,
                            segments: core::iter::once(syn::PathSegment {
                                ident: Ident::new("acc", Span::call_site()),
                                arguments: syn::PathArguments::None,
                            })
                            .collect(),
                        },
                    }),
                ]
                .into_iter()
                .collect(),
            }),
            None,
        ),
    ]
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
            transitions: BTreeMap::arbitrary(g)
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
                self.transitions
                    .iter()
                    .map(|(k, &(dst, _))| (k.clone(), dst))
                    .collect::<Vec<_>>(),
                self.accepting,
            )
                .shrink()
                .map(|(transitions, accepting)| Self {
                    transitions: transitions
                        .into_iter()
                        .map(|(k, v)| (k, (v, None)))
                        .collect(),
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
        state.transitions.retain(|_, &mut (index, _)| index < size);
    }
}
