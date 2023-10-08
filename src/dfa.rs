/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Deterministic finite automata.

#![cfg_attr(test, allow(dead_code))] // <-- FIXME

use crate::{call::Call, nfa, Expression};
use core::{
    cmp::{Ordering, Reverse},
    fmt::{self, Debug, Display},
    iter::once,
    slice::Iter,
};
use proc_macro2::{Span, TokenStream};
use std::{
    collections::{btree_map::Entry, BTreeMap, BTreeSet, BinaryHeap},
    vec::IntoIter,
};
use syn::{
    punctuated::Punctuated,
    token::{Brace, Bracket, Paren},
    AttrStyle, Attribute, Expr, ExprBlock, ExprCall, ExprIf, ExprLet, ExprLit, ExprMacro,
    ExprMatch, ExprMethodCall, ExprPath, ExprRange, ExprReturn, ExprUnsafe, FnArg, GenericArgument,
    GenericParam, Generics, Ident, Item, ItemFn, ItemMod, Lit, LitInt, LitStr, MacroDelimiter,
    Meta, MetaList, MetaNameValue, Pat, PatIdent, PatOr, PatParen, PatTupleStruct, PatType,
    PatWild, Path, PathArguments, PathSegment, RangeLimits, ReturnType, Signature, Stmt, Token,
    TraitBound, TraitBoundModifier, Type, TypeParam, TypeParamBound, TypePath, TypeReference,
    Visibility,
    __private::ToTokens,
};

#[cfg(feature = "quickcheck")]
use quickcheck::*;

/// Subset of states (by their index).
type Subset = BTreeSet<usize>;

/// From a single state, all tokens and the transitions each would induce.
type Transitions<I> = BTreeMap<I, Transition>;

/// A single edge triggered by a token.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct Transition {
    /// Destination state.
    pub(crate) dst: usize,
    /// Function (or none) to call on this edge.
    pub(crate) call: Call,
}

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
    pub(crate) transitions: Transitions<I>,
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
                Some(&Transition { dst, .. }) => state = dst,
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

    /// Remove all calls (set them to `None`).
    #[inline]
    #[must_use]
    #[cfg(test)]
    pub(crate) fn remove_calls(self) -> Self {
        Self {
            states: self.states.into_iter().map(State::remove_calls).collect(),
            ..self
        }
    }

    /// Generalize to an identical NFA.
    #[inline]
    #[must_use]
    pub fn generalize(&self) -> crate::Parser<I> {
        crate::Parser {
            states: self
                .states
                .iter()
                .map(|state| nfa::State {
                    epsilon: Subset::new(),
                    non_epsilon: state
                        .transitions
                        .iter()
                        .map(|(token, &Transition { dst, ref call })| {
                            (
                                token.clone(),
                                nfa::Transition {
                                    dsts: once(dst).collect(),
                                    call: call.clone(),
                                },
                            )
                        })
                        .collect(),
                    accepting: state.accepting,
                })
                .collect(),
            initial: once(self.initial).collect(),
        }
    }

    /// Randomly generate inputs that are all guaranteed to be accepted.
    /// NOTE: returns an infinite iterator! `for input in automaton.fuzz()?` will loop forever . . .
    /// # Errors
    /// If this automaton never accepts any input.
    #[inline]
    pub fn fuzz(&self) -> Result<crate::Fuzzer<I>, crate::NeverAccepts>
    where
        I: Debug,
    {
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
            for (token, &Transition { dst, .. }) in &state.transitions {
                if let Entry::Vacant(entry) = cache.entry(dst) {
                    entry.insert(cached.clone()).push(token.clone());
                    queue.push(Reverse(CmpFirst(distance.saturating_add(1), dst)));
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
    #[allow(clippy::too_many_lines)]
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
                Item::Fn(ast_fn),
                Item::Mod(ast_mod),
                Item::Fn(rev_fn),
                Item::Mod(rev_mod),
            ],
        })
    }

    /// Print as a set of Rust source-code functions.
    #[inline]
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn to_ast(&self, name: &str) -> (ItemFn, ItemMod)
    where
        I: Expression,
    {
        let generics = Generics {
            lt_token: Some(Token!(<)(Span::call_site())),
            params: once(GenericParam::Type(TypeParam {
                attrs: vec![],
                ident: Ident::new("I", Span::call_site()),
                colon_token: Some(Token!(:)(Span::call_site())),
                bounds: once(TypeParamBound::Trait(TraitBound {
                    paren_token: None,
                    modifier: TraitBoundModifier::None,
                    lifetimes: None,
                    path: Path {
                        leading_colon: None,
                        segments: once(PathSegment {
                            ident: Ident::new("Iterator", Span::call_site()),
                            arguments: PathArguments::AngleBracketed(
                                syn::AngleBracketedGenericArguments {
                                    colon2_token: None,
                                    lt_token: Token!(<)(Span::call_site()),
                                    args: once(GenericArgument::AssocType(syn::AssocType {
                                        ident: Ident::new("Item", Span::call_site()),
                                        generics: None,
                                        eq_token: Token!(=)(Span::call_site()),
                                        ty: I::to_type(),
                                    }))
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
        let states = ItemMod {
            attrs: vec![
                Attribute {
                    pound_token: Token!(#)(Span::call_site()),
                    style: AttrStyle::Outer,
                    bracket_token: Bracket::default(),
                    meta: Meta::List(MetaList {
                        path: Path {
                            leading_colon: None,
                            segments: once(PathSegment {
                                ident: Ident::new("allow", Span::call_site()),
                                arguments: PathArguments::None,
                            })
                            .collect(),
                        },
                        delimiter: MacroDelimiter::Paren(Paren::default()),
                        tokens: Ident::new("non_snake_case", Span::call_site()).into_token_stream(),
                    }),
                },
                Attribute {
                    pound_token: Token!(#)(Span::call_site()),
                    style: AttrStyle::Outer,
                    bracket_token: Bracket::default(),
                    meta: Meta::List(MetaList {
                        path: Path {
                            leading_colon: None,
                            segments: once(PathSegment {
                                ident: Ident::new("allow", Span::call_site()),
                                arguments: PathArguments::None,
                            })
                            .collect(),
                        },
                        delimiter: MacroDelimiter::Paren(Paren::default()),
                        tokens: Ident::new("non_camel_case_types", Span::call_site())
                            .into_token_stream(),
                    }),
                },
                Attribute {
                    pound_token: Token!(#)(Span::call_site()),
                    style: AttrStyle::Outer,
                    bracket_token: Bracket::default(),
                    meta: Meta::List(MetaList {
                        path: Path {
                            leading_colon: None,
                            segments: once(PathSegment {
                                ident: Ident::new("allow", Span::call_site()),
                                arguments: PathArguments::None,
                            })
                            .collect(),
                        },
                        delimiter: MacroDelimiter::Paren(Paren::default()),
                        tokens: Ident::new("unused_parens", Span::call_site()).into_token_stream(),
                    }),
                },
                Attribute {
                    pound_token: Token!(#)(Span::call_site()),
                    style: AttrStyle::Outer,
                    bracket_token: Bracket::default(),
                    meta: Meta::List(MetaList {
                        path: Path {
                            leading_colon: None,
                            segments: once(PathSegment {
                                ident: Ident::new("allow", Span::call_site()),
                                arguments: PathArguments::None,
                            })
                            .collect(),
                        },
                        delimiter: MacroDelimiter::Paren(Paren::default()),
                        tokens: Ident::new("unused_variables", Span::call_site())
                            .into_token_stream(),
                    }),
                },
            ],
            vis: Visibility::Restricted(syn::VisRestricted {
                pub_token: Token!(pub)(Span::call_site()),
                paren_token: Paren::default(),
                in_token: None,
                path: Box::new(Path {
                    leading_colon: None,
                    segments: once(PathSegment {
                        ident: Ident::new("crate", Span::call_site()),
                        arguments: PathArguments::None,
                    })
                    .collect(),
                }),
            }),
            unsafety: None,
            mod_token: Token!(mod)(Span::call_site()),
            ident: Ident::new(&format!("{name}_states"), Span::call_site()),
            content: Some((
                Brace::default(),
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
            ItemFn {
                attrs: vec![Attribute {
                    pound_token: Token!(#)(Span::call_site()),
                    style: AttrStyle::Outer,
                    bracket_token: Bracket::default(),
                    meta: Meta::List(MetaList {
                        path: Path {
                            leading_colon: None,
                            segments: once(PathSegment {
                                ident: Ident::new("inline", Span::call_site()),
                                arguments: PathArguments::None,
                            })
                            .collect(),
                        },
                        delimiter: MacroDelimiter::Paren(Paren::default()),
                        tokens: Ident::new("always", Span::call_site()).into_token_stream(),
                    }),
                }],
                block: Box::new(syn::Block {
                    brace_token: Brace::default(),
                    stmts: vec![Stmt::Expr(
                        Expr::Call(ExprCall {
                            attrs: vec![],
                            func: Box::new(Expr::Path(ExprPath {
                                attrs: vec![],
                                qself: None,
                                path: Path {
                                    leading_colon: None,
                                    segments: [
                                        PathSegment {
                                            ident: Ident::new(
                                                &format!("{name}_states"),
                                                Span::call_site(),
                                            ),
                                            arguments: PathArguments::None,
                                        },
                                        PathSegment {
                                            ident: Ident::new(
                                                &format!("s{}", self.initial),
                                                Span::call_site(),
                                            ),
                                            arguments: PathArguments::None,
                                        },
                                    ]
                                    .into_iter()
                                    .collect(),
                                },
                            })),
                            paren_token: Paren::default(),
                            args: [
                                Expr::Path(ExprPath {
                                    attrs: vec![],
                                    qself: None,
                                    path: Path {
                                        leading_colon: None,
                                        segments: once(PathSegment {
                                            ident: Ident::new("i", Span::call_site()),
                                            arguments: PathArguments::None,
                                        })
                                        .collect(),
                                    },
                                }),
                                Expr::Call(ExprCall {
                                    attrs: vec![],
                                    func: Box::new(Expr::Path(ExprPath {
                                        attrs: vec![],
                                        qself: None,
                                        path: config_path(name, "initial"),
                                    })),
                                    paren_token: Paren::default(),
                                    args: Punctuated::new(),
                                }),
                            ]
                            .into_iter()
                            .collect(),
                        }),
                        None,
                    )],
                }),
                sig: Signature {
                    constness: None,
                    asyncness: None,
                    unsafety: None,
                    abi: None,
                    fn_token: Token!(fn)(Span::call_site()),
                    ident: Ident::new(name, Span::call_site()),
                    generics,
                    paren_token: Paren::default(),
                    inputs: once(FnArg::Typed(PatType {
                        attrs: vec![],
                        pat: Box::new(Pat::Ident(PatIdent {
                            attrs: vec![],
                            by_ref: None,
                            mutability: None,
                            ident: Ident::new("i", Span::call_site()),
                            subpat: None,
                        })),
                        colon_token: Token!(:)(Span::call_site()),
                        ty: Box::new(Type::Path(TypePath {
                            qself: None,
                            path: Path {
                                leading_colon: None,
                                segments: once(PathSegment {
                                    ident: Ident::new("I", Span::call_site()),
                                    arguments: PathArguments::None,
                                })
                                .collect(),
                            },
                        })),
                    }))
                    .collect(),
                    variadic: None,
                    output: ReturnType::Type(
                        Token!(->)(Span::call_site()),
                        Box::new(Type::Path(TypePath {
                            qself: None,
                            path: Path {
                                leading_colon: None,
                                segments: once(PathSegment {
                                    ident: Ident::new("Option", Span::call_site()),
                                    arguments: PathArguments::AngleBracketed(
                                        syn::AngleBracketedGenericArguments {
                                            colon2_token: None,
                                            lt_token: Token!(<)(Span::call_site()),
                                            args: once(GenericArgument::Type(output_type(name)))
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
                vis: Visibility::Public(Token!(pub)(Span::call_site())),
            },
            states,
        )
    }

    /// Print as a set of Rust source-code functions.
    #[inline]
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn to_fuzz_ast(&self, name: &str) -> (ItemFn, ItemMod)
    where
        I: Expression,
    {
        let generics = Generics {
            lt_token: Some(Token!(<)(Span::call_site())),
            params: once(GenericParam::Type(TypeParam {
                attrs: vec![],
                ident: Ident::new("R", Span::call_site()),
                colon_token: Some(Token!(:)(Span::call_site())),
                bounds: once(TypeParamBound::Trait(TraitBound {
                    paren_token: None,
                    modifier: TraitBoundModifier::None,
                    lifetimes: None,
                    path: Path {
                        leading_colon: None,
                        segments: [
                            PathSegment {
                                ident: Ident::new("rand", Span::call_site()),
                                arguments: PathArguments::None,
                            },
                            PathSegment {
                                ident: Ident::new("Rng", Span::call_site()),
                                arguments: PathArguments::None,
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
        let states = ItemMod {
            attrs: vec![
                Attribute {
                    pound_token: Token!(#)(Span::call_site()),
                    style: AttrStyle::Outer,
                    bracket_token: Bracket::default(),
                    meta: Meta::List(MetaList {
                        path: Path {
                            leading_colon: None,
                            segments: once(PathSegment {
                                ident: Ident::new("allow", Span::call_site()),
                                arguments: PathArguments::None,
                            })
                            .collect(),
                        },
                        delimiter: MacroDelimiter::Paren(Paren::default()),
                        tokens: Ident::new("non_snake_case", Span::call_site()).into_token_stream(),
                    }),
                },
                Attribute {
                    pound_token: Token!(#)(Span::call_site()),
                    style: AttrStyle::Outer,
                    bracket_token: Bracket::default(),
                    meta: Meta::List(MetaList {
                        path: Path {
                            leading_colon: None,
                            segments: once(PathSegment {
                                ident: Ident::new("allow", Span::call_site()),
                                arguments: PathArguments::None,
                            })
                            .collect(),
                        },
                        delimiter: MacroDelimiter::Paren(Paren::default()),
                        tokens: Ident::new("unused_mut", Span::call_site()).into_token_stream(),
                    }),
                },
            ],
            vis: Visibility::Inherited,
            unsafety: None,
            mod_token: Token!(mod)(Span::call_site()),
            ident: Ident::new(&format!("{name}_states"), Span::call_site()),
            content: Some((
                Brace::default(),
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
            ItemFn {
                attrs: vec![Attribute {
                    pound_token: Token!(#)(Span::call_site()),
                    style: AttrStyle::Outer,
                    bracket_token: Bracket::default(),
                    meta: Meta::Path(Path {
                        leading_colon: None,
                        segments: once(PathSegment {
                            ident: Ident::new("inline", Span::call_site()),
                            arguments: PathArguments::None,
                        })
                        .collect(),
                    }),
                }],
                vis: Visibility::Public(Token!(pub)(Span::call_site())),
                sig: Signature {
                    constness: None,
                    asyncness: None,
                    unsafety: None,
                    abi: None,
                    fn_token: Token!(fn)(Span::call_site()),
                    ident: Ident::new(name, Span::call_site()),
                    generics,
                    paren_token: Paren::default(),
                    inputs: once(FnArg::Typed(PatType {
                        attrs: vec![],
                        pat: Box::new(Pat::Ident(PatIdent {
                            attrs: vec![],
                            by_ref: None,
                            mutability: None,
                            ident: Ident::new("r", Span::call_site()),
                            subpat: None,
                        })),
                        colon_token: Token!(:)(Span::call_site()),
                        ty: Box::new(Type::Reference(TypeReference {
                            and_token: Token!(&)(Span::call_site()),
                            lifetime: None,
                            mutability: Some(Token!(mut)(Span::call_site())),
                            elem: Box::new(Type::Path(TypePath {
                                qself: None,
                                path: Path {
                                    leading_colon: None,
                                    segments: once(PathSegment {
                                        ident: Ident::new("R", Span::call_site()),
                                        arguments: PathArguments::None,
                                    })
                                    .collect(),
                                },
                            })),
                        })),
                    }))
                    .collect(),
                    variadic: None,
                    output: ReturnType::Type(
                        Token!(->)(Span::call_site()),
                        Box::new(Type::Path(TypePath {
                            qself: None,
                            path: Path {
                                leading_colon: None,
                                segments: once(PathSegment {
                                    ident: Ident::new("Vec", Span::call_site()),
                                    arguments: PathArguments::AngleBracketed(
                                        syn::AngleBracketedGenericArguments {
                                            colon2_token: None,
                                            lt_token: Token!(<)(Span::call_site()),
                                            args: once(GenericArgument::Type(I::to_type()))
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
                    brace_token: Brace::default(),
                    stmts: vec![
                        Stmt::Expr(
                            Expr::Let(ExprLet {
                                attrs: vec![],
                                let_token: Token!(let)(Span::call_site()),
                                pat: Box::new(Pat::Ident(PatIdent {
                                    attrs: vec![],
                                    by_ref: None,
                                    mutability: Some(Token!(mut)(Span::call_site())),
                                    ident: Ident::new("v", Span::call_site()),
                                    subpat: None,
                                })),
                                eq_token: Token!(=)(Span::call_site()),
                                expr: Box::new(Expr::Call(ExprCall {
                                    attrs: vec![],
                                    func: Box::new(Expr::Path(ExprPath {
                                        attrs: vec![],
                                        qself: None,
                                        path: Path {
                                            leading_colon: None,
                                            segments: [
                                                PathSegment {
                                                    ident: Ident::new(
                                                        &format!("{name}_states"),
                                                        Span::call_site(),
                                                    ),
                                                    arguments: PathArguments::None,
                                                },
                                                PathSegment {
                                                    ident: Ident::new(
                                                        &format!("s{}", self.initial),
                                                        Span::call_site(),
                                                    ),
                                                    arguments: PathArguments::None,
                                                },
                                            ]
                                            .into_iter()
                                            .collect(),
                                        },
                                    })),
                                    paren_token: Paren::default(),
                                    args: [
                                        Expr::Path(ExprPath {
                                            attrs: vec![],
                                            qself: None,
                                            path: Path {
                                                leading_colon: None,
                                                segments: once(PathSegment {
                                                    ident: Ident::new("r", Span::call_site()),
                                                    arguments: PathArguments::None,
                                                })
                                                .collect(),
                                            },
                                        }),
                                        Expr::Macro(ExprMacro {
                                            attrs: vec![],
                                            mac: syn::Macro {
                                                path: Path {
                                                    leading_colon: None,
                                                    segments: once(PathSegment {
                                                        ident: Ident::new("vec", Span::call_site()),
                                                        arguments: PathArguments::None,
                                                    })
                                                    .collect(),
                                                },
                                                bang_token: Token!(!)(Span::call_site()),
                                                delimiter: MacroDelimiter::Bracket(
                                                    Bracket::default(),
                                                ),
                                                tokens: TokenStream::new(),
                                            },
                                        }),
                                    ]
                                    .into_iter()
                                    .collect(),
                                })),
                            }),
                            Some(Token!(;)(Span::call_site())),
                        ),
                        Stmt::Expr(
                            Expr::MethodCall(ExprMethodCall {
                                attrs: vec![],
                                receiver: Box::new(Expr::Path(ExprPath {
                                    attrs: vec![],
                                    qself: None,
                                    path: Path {
                                        leading_colon: None,
                                        segments: once(PathSegment {
                                            ident: Ident::new("v", Span::call_site()),
                                            arguments: PathArguments::None,
                                        })
                                        .collect(),
                                    },
                                })),
                                dot_token: Token!(.)(Span::call_site()),
                                method: Ident::new("reverse", Span::call_site()),
                                turbofish: None,
                                paren_token: Paren::default(),
                                args: Punctuated::new(),
                            }),
                            Some(Token!(;)(Span::call_site())),
                        ),
                        Stmt::Expr(
                            Expr::Path(ExprPath {
                                attrs: vec![],
                                qself: None,
                                path: Path {
                                    leading_colon: None,
                                    segments: once(PathSegment {
                                        ident: Ident::new("v", Span::call_site()),
                                        arguments: PathArguments::None,
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
pub(crate) struct CmpFirst<A: Ord, B>(pub(crate) A, pub(crate) B);

impl<A: Ord, B> PartialEq for CmpFirst<A, B> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<A: Ord, B> Eq for CmpFirst<A, B> {}

impl<A: Ord, B> PartialOrd for CmpFirst<A, B> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<A: Ord, B> Ord for CmpFirst<A, B> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<I: Clone + Ord> IntoIterator for Graph<I> {
    type Item = State<I>;
    type IntoIter = IntoIter<State<I>>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.states.into_iter()
    }
}

impl<'a, I: Clone + Ord> IntoIterator for &'a Graph<I> {
    type Item = &'a State<I>;
    type IntoIter = Iter<'a, State<I>>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.states.iter()
    }
}

impl<I: Clone + Ord + Expression> Display for Graph<I> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Initial state: {}", self.initial)?;
        for (i, state) in self.states.iter().enumerate() {
            write!(f, "State {i} {state}")?;
        }
        Ok(())
    }
}

impl<I: Clone + Ord + Expression> Display for State<I> {
    #[inline]
    #[allow(clippy::use_debug)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "({}accepting):",
            if self.accepting { "" } else { "NOT " }
        )?;
        for (input, &Transition { dst, ref call }) in &self.transitions {
            writeln!(f, "    {input:?} --> {dst} >>= {call:?}")?;
        }
        Ok(())
    }
}

/// User-defined config module after build time.
#[inline]
pub(crate) fn config_path(name: &str, destination: &str) -> Path {
    Path {
        leading_colon: None,
        segments: [
            PathSegment {
                ident: Ident::new("crate", Span::call_site()),
                arguments: PathArguments::None,
            },
            PathSegment {
                ident: Ident::new("inator_config", Span::call_site()),
                arguments: PathArguments::None,
            },
            PathSegment {
                ident: Ident::new(name, Span::call_site()),
                arguments: PathArguments::None,
            },
            PathSegment {
                ident: Ident::new(destination, Span::call_site()),
                arguments: PathArguments::None,
            },
        ]
        .into_iter()
        .collect(),
    }
}

/// Reference the user-defined output type after build time.
#[inline]
fn output_type(name: &str) -> Type {
    Type::Path(TypePath {
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
    pub(crate) fn transition(&self, input: &I) -> Option<&Transition> {
        self.transitions.get(input)
    }

    /// Remove all calls (set them to `None`).
    #[inline]
    #[must_use]
    #[cfg(test)]
    pub(crate) fn remove_calls(self) -> Self {
        Self {
            transitions: self
                .transitions
                .into_iter()
                .map(|(token, transition)| (token, transition.remove_calls()))
                .collect(),
            ..self
        }
    }

    /// Print as a Rust source-code function.
    #[inline]
    #[allow(clippy::too_many_lines)]
    pub fn to_source(
        &self,
        index: usize,
        name: &str,
        generics: Generics,
        minimal_input: &[I],
    ) -> Vec<Item>
    where
        I: Expression,
    {
        let inverted = invert(&self.transitions);
        vec![Item::Fn(ItemFn {
            attrs: vec![
                Attribute {
                    pound_token: Token!(#)(Span::call_site()),
                    style: AttrStyle::Outer,
                    bracket_token: Bracket::default(),
                    meta: Meta::Path(Path {
                        leading_colon: None,
                        segments: once(PathSegment {
                            ident: Ident::new("inline", Span::call_site()),
                            arguments: PathArguments::None,
                        })
                        .collect(),
                    }),
                },
                Attribute {
                    pound_token: Token!(#)(Span::call_site()),
                    style: AttrStyle::Outer,
                    bracket_token: Bracket::default(),
                    meta: Meta::NameValue(MetaNameValue {
                        path: Path {
                            leading_colon: None,
                            segments: once(PathSegment {
                                ident: Ident::new("doc", Span::call_site()),
                                arguments: PathArguments::None,
                            })
                            .collect(),
                        },
                        eq_token: Token!(=)(Span::call_site()),
                        value: Expr::Lit(ExprLit {
                            attrs: vec![],
                            lit: Lit::Str(LitStr::new(
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
            vis: Visibility::Public(Token!(pub)(Span::call_site())),
            sig: Signature {
                constness: None,
                asyncness: None,
                unsafety: None,
                abi: None,
                fn_token: Token!(fn)(Span::call_site()),
                ident: Ident::new(&format!("s{index}"), Span::call_site()),
                generics,
                paren_token: Paren::default(),
                inputs: [
                    FnArg::Typed(PatType {
                        attrs: vec![],
                        pat: Box::new(Pat::Ident(PatIdent {
                            attrs: vec![],
                            by_ref: None,
                            mutability: Some(Token!(mut)(Span::call_site())),
                            ident: Ident::new("i", Span::call_site()),
                            subpat: None,
                        })),
                        colon_token: Token!(:)(Span::call_site()),
                        ty: Box::new(Type::Path(TypePath {
                            qself: None,
                            path: Path {
                                leading_colon: None,
                                segments: once(PathSegment {
                                    ident: Ident::new("I", Span::call_site()),
                                    arguments: PathArguments::None,
                                })
                                .collect(),
                            },
                        })),
                    }),
                    FnArg::Typed(PatType {
                        attrs: vec![],
                        pat: Box::new(Pat::Ident(PatIdent {
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
                output: ReturnType::Type(
                    Token!(->)(Span::call_site()),
                    Box::new(Type::Path(TypePath {
                        qself: None,
                        path: Path {
                            leading_colon: None,
                            segments: once(PathSegment {
                                ident: Ident::new("Option", Span::call_site()),
                                arguments: PathArguments::AngleBracketed(
                                    syn::AngleBracketedGenericArguments {
                                        colon2_token: None,
                                        lt_token: Token!(<)(Span::call_site()),
                                        args: once(GenericArgument::Type(output_type(name)))
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
                brace_token: Brace::default(),
                stmts: vec![Stmt::Expr(
                    Expr::Match(ExprMatch {
                        attrs: vec![],
                        match_token: Token!(match)(Span::call_site()),
                        expr: Box::new(Expr::MethodCall(ExprMethodCall {
                            attrs: vec![],
                            receiver: Box::new(Expr::Path(ExprPath {
                                attrs: vec![],
                                qself: None,
                                path: Path {
                                    leading_colon: None,
                                    segments: once(PathSegment {
                                        ident: Ident::new("i", Span::call_site()),
                                        arguments: PathArguments::None,
                                    })
                                    .collect(),
                                },
                            })),
                            dot_token: Token!(.)(Span::call_site()),
                            method: Ident::new("next", Span::call_site()),
                            turbofish: None,
                            paren_token: Paren::default(),
                            args: Punctuated::new(),
                        })),
                        brace_token: Brace::default(),
                        arms: {
                            let mut v = vec![];
                            for (
                                &&Transition {
                                    dst,
                                    call: ref fn_name,
                                },
                                inputs,
                            ) in &inverted
                            {
                                let sdst = format!("s{dst}");
                                v.push(syn::Arm {
                                    attrs: vec![],
                                    pat: Pat::TupleStruct(PatTupleStruct {
                                        attrs: vec![],
                                        qself: None,
                                        path: Path {
                                            leading_colon: None,
                                            segments: once(PathSegment {
                                                ident: Ident::new("Some", Span::call_site()),
                                                arguments: PathArguments::None,
                                            })
                                            .collect(),
                                        },
                                        paren_token: Paren::default(),
                                        elems: once(Pat::Ident(PatIdent {
                                            attrs: vec![],
                                            by_ref: None,
                                            mutability: None,
                                            ident: Ident::new("token", Span::call_site()),
                                            subpat: Some((
                                                Token!(@)(Span::call_site()),
                                                Box::new(Pat::Paren(PatParen {
                                                    attrs: vec![],
                                                    paren_token: Paren::default(),
                                                    pat: Box::new(Pat::Or(PatOr {
                                                        attrs: vec![],
                                                        leading_vert: None,
                                                        cases: inputs
                                                            .iter()
                                                            .map(|input| input.to_pattern())
                                                            .collect(),
                                                    })),
                                                })),
                                            )),
                                        }))
                                        .collect(),
                                    }),
                                    guard: None,
                                    fat_arrow_token: Token!(=>)(Span::call_site()),
                                    body: Box::new(Expr::Call(ExprCall {
                                        attrs: vec![],
                                        func: Box::new(Expr::Path(ExprPath {
                                            attrs: vec![],
                                            qself: None,
                                            path: Path {
                                                leading_colon: None,
                                                segments: once(PathSegment {
                                                    ident: Ident::new(&sdst, Span::call_site()),
                                                    arguments: PathArguments::None,
                                                })
                                                .collect(),
                                            },
                                        })),
                                        paren_token: Paren::default(),
                                        args: [
                                            Expr::Path(ExprPath {
                                                attrs: vec![],
                                                qself: None,
                                                path: Path {
                                                    leading_colon: None,
                                                    segments: once(PathSegment {
                                                        ident: Ident::new("i", Span::call_site()),
                                                        arguments: PathArguments::None,
                                                    })
                                                    .collect(),
                                                },
                                            }),
                                            fn_name.to_expr(),
                                        ]
                                        .into_iter()
                                        .collect(),
                                    })),
                                    comma: Some(Token!(,)(Span::call_site())),
                                });
                            }
                            v.push(syn::Arm {
                                attrs: vec![],
                                pat: Pat::Path(ExprPath {
                                    attrs: vec![],
                                    qself: None,
                                    path: Path {
                                        leading_colon: None,
                                        segments: once(PathSegment {
                                            ident: Ident::new("None", Span::call_site()),
                                            arguments: PathArguments::None,
                                        })
                                        .collect(),
                                    },
                                }),
                                guard: None,
                                fat_arrow_token: Token!(=>)(Span::call_site()),
                                body: Box::new(if self.accepting {
                                    Expr::Call(ExprCall {
                                        attrs: vec![],
                                        func: Box::new(Expr::Path(ExprPath {
                                            attrs: vec![],
                                            qself: None,
                                            path: Path {
                                                leading_colon: None,
                                                segments: once(PathSegment {
                                                    ident: Ident::new("Some", Span::call_site()),
                                                    arguments: PathArguments::None,
                                                })
                                                .collect(),
                                            },
                                        })),
                                        paren_token: Paren::default(),
                                        args: once(Expr::Path(ExprPath {
                                            attrs: vec![],
                                            qself: None,
                                            path: Path {
                                                leading_colon: None,
                                                segments: once(PathSegment {
                                                    ident: Ident::new("acc", Span::call_site()),
                                                    arguments: PathArguments::None,
                                                })
                                                .collect(),
                                            },
                                        }))
                                        .collect(),
                                    })
                                } else {
                                    Expr::Path(ExprPath {
                                        attrs: vec![],
                                        qself: None,
                                        path: Path {
                                            leading_colon: None,
                                            segments: once(PathSegment {
                                                ident: Ident::new("None", Span::call_site()),
                                                arguments: PathArguments::None,
                                            })
                                            .collect(),
                                        },
                                    })
                                }),
                                comma: Some(Token!(,)(Span::call_site())),
                            });
                            v.push(syn::Arm {
                                attrs: vec![],
                                pat: Pat::Wild(PatWild {
                                    attrs: vec![],
                                    underscore_token: Token!(_)(Span::call_site()),
                                }),
                                guard: None,
                                fat_arrow_token: Token!(=>)(Span::call_site()),
                                body: Box::new(Expr::Path(ExprPath {
                                    attrs: vec![],
                                    qself: None,
                                    path: Path {
                                        leading_colon: None,
                                        segments: once(PathSegment {
                                            ident: Ident::new("None", Span::call_site()),
                                            arguments: PathArguments::None,
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
        generics: Generics,
        minimal_input: &[I],
    ) -> Item
    where
        I: Expression,
    {
        let mut stmts = vec![];
        if self.accepting {
            stmts.push(Stmt::Expr(
                Expr::If(ExprIf {
                    attrs: vec![],
                    if_token: Token!(if)(Span::call_site()),
                    cond: Box::new(Expr::MethodCall(ExprMethodCall {
                        attrs: vec![],
                        receiver: Box::new(Expr::Path(ExprPath {
                            attrs: vec![],
                            qself: None,
                            path: Path {
                                leading_colon: None,
                                segments: once(PathSegment {
                                    ident: Ident::new("r", Span::call_site()),
                                    arguments: PathArguments::None,
                                })
                                .collect(),
                            },
                        })),
                        dot_token: Token!(.)(Span::call_site()),
                        method: Ident::new("gen", Span::call_site()),
                        turbofish: None,
                        paren_token: Paren::default(),
                        args: Punctuated::new(),
                    })),
                    then_branch: syn::Block {
                        brace_token: Brace::default(),
                        stmts: vec![Stmt::Expr(
                            Expr::Return(ExprReturn {
                                attrs: vec![],
                                return_token: Token!(return)(Span::call_site()),
                                expr: Some(Box::new(Expr::Path(ExprPath {
                                    attrs: vec![],
                                    qself: None,
                                    path: Path {
                                        leading_colon: None,
                                        segments: once(PathSegment {
                                            ident: Ident::new("acc", Span::call_site()),
                                            arguments: PathArguments::None,
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
            0 => stmts.push(Stmt::Expr(
                Expr::Call(ExprCall {
                    attrs: vec![],
                    func: Box::new(Expr::Path(ExprPath {
                        attrs: vec![],
                        qself: None,
                        path: Path {
                            leading_colon: None,
                            segments: once(PathSegment {
                                ident: Ident::new(&format!("s{initial}"), Span::call_site()),
                                arguments: PathArguments::None,
                            })
                            .collect(),
                        },
                    })),
                    paren_token: Paren::default(),
                    args: [
                        Expr::Path(ExprPath {
                            attrs: vec![],
                            qself: None,
                            path: Path {
                                leading_colon: None,
                                segments: once(PathSegment {
                                    ident: Ident::new("r", Span::call_site()),
                                    arguments: PathArguments::None,
                                })
                                .collect(),
                            },
                        }),
                        Expr::Macro(ExprMacro {
                            attrs: vec![],
                            mac: syn::Macro {
                                path: Path {
                                    leading_colon: None,
                                    segments: once(PathSegment {
                                        ident: Ident::new("vec", Span::call_site()),
                                        arguments: PathArguments::None,
                                    })
                                    .collect(),
                                },
                                bang_token: Token!(!)(Span::call_site()),
                                delimiter: MacroDelimiter::Bracket(Bracket::default()),
                                tokens: TokenStream::new(),
                            },
                        }),
                    ]
                    .into_iter()
                    .collect(),
                }),
                None,
            )),
            1 => stmts.extend({
                let (token, &Transition { dst, .. }) = unwrap!(self.transitions.first_key_value());
                fuzz_stmts(token, dst)
            }),
            _ => stmts.push(Stmt::Expr(
                Expr::Match(ExprMatch {
                    attrs: vec![],
                    match_token: Token!(match)(Span::call_site()),
                    expr: Box::new(Expr::MethodCall(ExprMethodCall {
                        attrs: vec![],
                        receiver: Box::new(Expr::Path(ExprPath {
                            attrs: vec![],
                            qself: None,
                            path: Path {
                                leading_colon: None,
                                segments: once(PathSegment {
                                    ident: Ident::new("r", Span::call_site()),
                                    arguments: PathArguments::None,
                                })
                                .collect(),
                            },
                        })),
                        dot_token: Token!(.)(Span::call_site()),
                        method: Ident::new("gen_range", Span::call_site()),
                        turbofish: None,
                        paren_token: Paren::default(),
                        args: once(Expr::Range(ExprRange {
                            attrs: vec![],
                            start: Some(Box::new(Expr::Lit(ExprLit {
                                attrs: vec![],
                                lit: Lit::Int(LitInt::new("0", Span::call_site())),
                            }))),
                            limits: RangeLimits::HalfOpen(Token!(..)(Span::call_site())),
                            end: Some(Box::new(Expr::Lit(ExprLit {
                                attrs: vec![],
                                lit: Lit::Int(LitInt::new(
                                    &format!("{}", self.transitions.len()),
                                    Span::call_site(),
                                )),
                            }))),
                        }))
                        .collect(),
                    })),
                    brace_token: Brace::default(),
                    arms: self
                        .transitions
                        .iter()
                        .enumerate()
                        .map(|(i, (token, &Transition { dst, .. }))| syn::Arm {
                            attrs: vec![],
                            pat: Pat::Lit(ExprLit {
                                attrs: vec![],
                                lit: Lit::Int(LitInt::new(&format!("{i}"), Span::call_site())),
                            }),
                            guard: None,
                            fat_arrow_token: Token!(=>)(Span::call_site()),
                            body: Box::new(Expr::Block(ExprBlock {
                                attrs: vec![],
                                label: None,
                                block: syn::Block {
                                    brace_token: Brace::default(),
                                    stmts: fuzz_stmts(token, dst),
                                },
                            })),
                            comma: None,
                        })
                        .chain(once(syn::Arm {
                            attrs: vec![],
                            pat: Pat::Wild(PatWild {
                                attrs: vec![],
                                underscore_token: Token!(_)(Span::call_site()),
                            }),
                            guard: None,
                            fat_arrow_token: Token!(=>)(Span::call_site()),
                            body: Box::new(Expr::Unsafe(ExprUnsafe {
                                attrs: vec![],
                                unsafe_token: Token!(unsafe)(Span::call_site()),
                                block: syn::Block {
                                    brace_token: Brace::default(),
                                    stmts: vec![Stmt::Expr(
                                        Expr::Call(ExprCall {
                                            attrs: vec![],
                                            func: Box::new(Expr::Path(ExprPath {
                                                attrs: vec![],
                                                qself: None,
                                                path: Path {
                                                    leading_colon: None,
                                                    segments: [
                                                        PathSegment {
                                                            ident: Ident::new(
                                                                "core",
                                                                Span::call_site(),
                                                            ),
                                                            arguments: PathArguments::None,
                                                        },
                                                        PathSegment {
                                                            ident: Ident::new(
                                                                "hint",
                                                                Span::call_site(),
                                                            ),
                                                            arguments: PathArguments::None,
                                                        },
                                                        PathSegment {
                                                            ident: Ident::new(
                                                                "unreachable_unchecked",
                                                                Span::call_site(),
                                                            ),
                                                            arguments: PathArguments::None,
                                                        },
                                                    ]
                                                    .into_iter()
                                                    .collect(),
                                                },
                                            })),
                                            paren_token: Paren::default(),
                                            args: Punctuated::new(),
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
        Item::Fn(ItemFn {
            attrs: vec![
                Attribute {
                    pound_token: Token!(#)(Span::call_site()),
                    style: AttrStyle::Outer,
                    bracket_token: Bracket::default(),
                    meta: Meta::Path(Path {
                        leading_colon: None,
                        segments: once(PathSegment {
                            ident: Ident::new("inline", Span::call_site()),
                            arguments: PathArguments::None,
                        })
                        .collect(),
                    }),
                },
                Attribute {
                    pound_token: Token!(#)(Span::call_site()),
                    style: AttrStyle::Outer,
                    bracket_token: Bracket::default(),
                    meta: Meta::NameValue(MetaNameValue {
                        path: Path {
                            leading_colon: None,
                            segments: once(PathSegment {
                                ident: Ident::new("doc", Span::call_site()),
                                arguments: PathArguments::None,
                            })
                            .collect(),
                        },
                        eq_token: Token!(=)(Span::call_site()),
                        value: Expr::Lit(ExprLit {
                            attrs: vec![],
                            lit: Lit::Str(LitStr::new(
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
            vis: Visibility::Public(Token!(pub)(Span::call_site())),
            sig: Signature {
                constness: None,
                asyncness: None,
                unsafety: None,
                abi: None,
                fn_token: Token!(fn)(Span::call_site()),
                ident: Ident::new(&format!("s{index}"), Span::call_site()),
                generics,
                paren_token: Paren::default(),
                inputs: [
                    FnArg::Typed(PatType {
                        attrs: vec![],
                        pat: Box::new(Pat::Ident(PatIdent {
                            attrs: vec![],
                            by_ref: None,
                            mutability: None,
                            ident: Ident::new("r", Span::call_site()),
                            subpat: None,
                        })),
                        colon_token: Token!(:)(Span::call_site()),
                        ty: Box::new(Type::Reference(TypeReference {
                            and_token: Token!(&)(Span::call_site()),
                            lifetime: None,
                            mutability: Some(Token!(mut)(Span::call_site())),
                            elem: Box::new(Type::Path(TypePath {
                                qself: None,
                                path: Path {
                                    leading_colon: None,
                                    segments: once(PathSegment {
                                        ident: Ident::new("R", Span::call_site()),
                                        arguments: PathArguments::None,
                                    })
                                    .collect(),
                                },
                            })),
                        })),
                    }),
                    FnArg::Typed(PatType {
                        attrs: vec![],
                        pat: Box::new(Pat::Ident(PatIdent {
                            attrs: vec![],
                            by_ref: None,
                            mutability: Some(Token!(mut)(Span::call_site())),
                            ident: Ident::new("acc", Span::call_site()),
                            subpat: None,
                        })),
                        colon_token: Token!(:)(Span::call_site()),
                        ty: Box::new(Type::Path(TypePath {
                            qself: None,
                            path: Path {
                                leading_colon: None,
                                segments: once(PathSegment {
                                    ident: Ident::new("Vec", Span::call_site()),
                                    arguments: PathArguments::AngleBracketed(
                                        syn::AngleBracketedGenericArguments {
                                            colon2_token: None,
                                            lt_token: Token!(<)(Span::call_site()),
                                            args: once(GenericArgument::Type(I::to_type()))
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
                output: ReturnType::Type(
                    Token!(->)(Span::call_site()),
                    Box::new(Type::Path(TypePath {
                        qself: None,
                        path: Path {
                            leading_colon: None,
                            segments: once(PathSegment {
                                ident: Ident::new("Vec", Span::call_site()),
                                arguments: PathArguments::AngleBracketed(
                                    syn::AngleBracketedGenericArguments {
                                        colon2_token: None,
                                        lt_token: Token!(<)(Span::call_site()),
                                        args: once(GenericArgument::Type(I::to_type())).collect(),
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
                brace_token: Brace::default(),
                stmts,
            }),
        })
    }
}

/// When writing a fuzzer function, append this character then jump to this other state.
fn fuzz_stmts<E: Expression>(token: &E, dst: usize) -> Vec<Stmt> {
    vec![
        Stmt::Expr(
            Expr::MethodCall(ExprMethodCall {
                attrs: vec![],
                receiver: Box::new(Expr::Path(ExprPath {
                    attrs: vec![],
                    qself: None,
                    path: Path {
                        leading_colon: None,
                        segments: once(PathSegment {
                            ident: Ident::new("acc", Span::call_site()),
                            arguments: PathArguments::None,
                        })
                        .collect(),
                    },
                })),
                dot_token: Token!(.)(Span::call_site()),
                method: Ident::new("push", Span::call_site()),
                turbofish: None,
                paren_token: Paren::default(),
                args: once(token.to_expr()).collect(),
            }),
            Some(Token!(;)(Span::call_site())),
        ),
        Stmt::Expr(
            Expr::Call(ExprCall {
                attrs: vec![],
                func: Box::new(Expr::Path(ExprPath {
                    attrs: vec![],
                    qself: None,
                    path: Path {
                        leading_colon: None,
                        segments: once(PathSegment {
                            ident: Ident::new(&format!("s{dst}"), Span::call_site()),
                            arguments: PathArguments::None,
                        })
                        .collect(),
                    },
                })),
                paren_token: Paren::default(),
                args: [
                    Expr::Path(ExprPath {
                        attrs: vec![],
                        qself: None,
                        path: Path {
                            leading_colon: None,
                            segments: once(PathSegment {
                                ident: Ident::new("r", Span::call_site()),
                                arguments: PathArguments::None,
                            })
                            .collect(),
                        },
                    }),
                    Expr::Path(ExprPath {
                        attrs: vec![],
                        qself: None,
                        path: Path {
                            leading_colon: None,
                            segments: once(PathSegment {
                                ident: Ident::new("acc", Span::call_site()),
                                arguments: PathArguments::None,
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

impl Transition {
    /// Remove all calls (set them to `None`).
    #[inline]
    #[must_use]
    #[cfg(test)]
    pub(crate) fn remove_calls(self) -> Self {
        Self {
            call: Call::Pass,
            ..self
        }
    }
}

#[cfg(feature = "quickcheck")]
impl<I: Ord + Arbitrary> Arbitrary for Graph<I> {
    #[inline]
    fn arbitrary(g: &mut Gen) -> Self {
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
impl<I: Ord + Arbitrary> Arbitrary for State<I> {
    #[inline]
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            transitions: BTreeMap::arbitrary(g)
                .into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        Transition {
                            dst: v,
                            call: Arbitrary::arbitrary(g),
                        },
                    )
                })
                .collect(),
            accepting: Arbitrary::arbitrary(g),
        }
    }

    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (
                self.transitions
                    .iter()
                    .map(|(k, &Transition { dst, ref call })| (k.clone(), dst, call.clone()))
                    .collect::<Vec<_>>(),
                self.accepting,
            )
                .shrink()
                .map(|(transitions, accepting)| Self {
                    transitions: transitions
                        .into_iter()
                        .map(|(token, dst, call)| (token, Transition { dst, call }))
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
        state
            .transitions
            .retain(|_, &mut Transition { dst, .. }| dst < size);
    }
}
