# `inator`: An evil parsing library.
## You supply the evil plan; we supply the _**-inator**_.

![Portrait of the eminent Dr. Heinz Doofenshmirtz](http://images6.fanpop.com/image/polls/1198000/1198459_1364687083851_full.jpg)

🚧 Development still ongoing! 🚧

## TL;DR

We ask for a specification and turn it into a graph where we know exactly what might happen at each step.
We then ruthlessly cut things that won't happen, combine identical things, and output the result as a Rust file.
Compile this with the rest of your code and, _voila!_, you've got a hand-rolled zero-copy parser.

## The long explanation

Given a spec like "`AAAAAAAAB` or `AAAAAAAAC`", we'd like to write a parser that takes the common-sense route:
blow through all the `A`s as a group, _then_ match on the `B` or `C` once.
This has several advantages:
- We never rewind the tape, so we don't need to store any previous input.
- In the limit, it's double as fast as trying one, then trying the other. If we added a third case, it'd be 3x. You get the point.
Yet, as far as I know, no existing parsing library even tries to optimize these cases, and for good reason: it's _hard_.

The problem with parsers is that, the more you can do with them, the less you can say about them before they run.

Regular languages are great to reason about, but they can't even parse parentheses. For a programming language, they're totally out of the running.

Context-free grammars are on the other end of the spectrum:
they're almost always all you need in practice, but there's no algorithm to compute the union of two pushdown automata, so you can't ask for "`AAAAAAAAB` or `AAAAAAAAC`" at all.
They're also just not closed on intersection, so there _can be_ no algorithm to require both A and B to accept an input C.

Visibly pushdown automata, which you've probably never heard of, bridge this gap:
Each symbol is permanently and universally associated with either pushing to the stack, popping from it, or not touching it.
They're closed under union and intersection, which is great, but the partitioning of input characters runs into some snags:
e.g. you can't use `<` both to open angle brackets and to compute a less-than relation.
(Also, shoutout to my professor Rajeev Alur, who—I later learned—co-invented these!)

In this library, we've gone totally opportunistic: just act like a pushdown automata until we hit an impossible intersection, then complain.
Every case I know of in which intersection fails has nothing to do with parsing, but if it is a limitation in practice for you, please let me know.

## What does this whole process look like?

Surprisingly, it looks a lot like just writing down what you want. Here's the definition for "put this thing in parentheses":

```rust
pub fn parenthesized(p: Parser<char>) -> Parser<char> {
    ignore('(') + p + ignore(')')
}
```

So, if `p` is a parser that accepts `ABC`, then `parenthesized(p)` will accept `(ABC)`. Simple as that.

The key idea here is that ***parsers are data***, and you can pass them around, modify them, and combine them just like anything else.

See `examples/tuple` for a well-annotated crate that reads characters from a tuple representation (`()`, `(A,)`, or `(A, B, C, ...)`), ignoring whitespace.
In that example, the entire parser could be defined in a single line, but I split it up to illustrate, first, _that you can do that_—you don't have to have a once-and-for-all megaparser—and, two, to explain as much about using the library as I can in detail.

Other crates in `examples` extend the same technique to more complex parsers (e.g. phone numbers and email addresses).

## Anything else cool you can do?

Yes! Since we're really just riding on top of a decision-problem automaton, you can take a specification and invert it to produce an infinite stream of random strings that are all guaranteed to be parsed correctly.
If you're writing a language, this means _automatically generating all possible valid source files_.

And, you guessed it, this gets compiled down to Rust source as well, so your property-tests can be ridiculously effective.

## Why not other parsing libraries?

Please try other parsing libraries! This is _my_ favorite, mostly becuase it's pure Rust with zero macros, no new syntax, zero input copying, parsers as data, automatic input generation, and—well–I wrote it, but I'm not too familiar with other libraries, so I can't in good faith recommend this one too highly.

My primary goal was a personal tool, but it turned out much better than I expected, so I'd love to see you use it and get your feedback!

## Acknowledgments

First off, to Haskell's parsing libraries (and UPenn's Haskell course) for showing me that parsers can even work this way. Most of the syntax is inspired by Haskell.

Second, to Rajeev Alur and Penn's CIS 262 for formally introducing me to nondeterministic finite automata.

Third, to Rust.
