# `inator`: a revolutionary parsing library
## You supply the evil plan; we supply the _**-inator!**_
### or, Provably Optimal Zero-Copy Parsers with Nondeterministic Finite Automata

![Portrait of the eminent Dr. Heinz Doofenshmirtz](http://images6.fanpop.com/image/polls/1198000/1198459_1364687083851_full.jpg)

## Why?
Computability theory has known for ages that every decision problem (anything with a yes/no answer) has a unique and probably optimal representation.

This is theoretically awesome and powers regular-expression searchers like `grep`, but—as far as I'm aware—anything more than a decision problem is theoretically not optimizable.

## How?
This library uses the same high-level machinery as tools like `grep` but augmented with some extra goods:

Given a specification like "parse A after B" or "put this thing in parentheses" (and infinite combinations of similar things), we write a graph with a set of initial nodes, a set of accepting nodes, and some directed edges marked with input characters.
From this, the computability theory stuff kicks in: we can minimize this to a graph with the provably minimal number of states that will tell us "is this valid syntax or not?"

Then, although we can't provably optimize _your_ code, we let you write functions that effectively _hitch a ride_ on the optimal decision graph we just made.
Specifically, each edge—that is, each time you said "parse this token"—you can call a function on the data so far, using that token, that returns the next state.

Last of all, you can convert this optimal graph to a _Rust source file_: not only can we prove the implementation is optimal, we can leverage Rust's binary optimizer to make it run literally as fast as possible.

## What does this whole process look like?

Surprisingly, it looks a lot like just writing down what you want. Here's the definition for "put this thing in parentheses":

```rust
pub fn parenthesized(p: Parser<char>) -> Parser<char> {
    ignore('(') + p + ignore(')')
}
```

So, if `p` is a parser that accepts `ABC`, then `parenthesized(p)` will accept `(ABC)`. Simple as that.

The key idea here is that ***parsers are data***, and you can pass them around, modify them, and combine them just like anything else.

See the `example` folder for a working self-contained crate using `inator` that reads characters from a tuple representation (`()`, `(A,)`, or `(A, B, C, ...)`), ignoring whitespace.
In that example, the entire parser could be defined in a single line, but I split it up to illustrate, first, _that you can do that_—you don't have to have a once-and-for-all megaparser—and, two, to explain as much about using the library as I can in detail.

## Anything else cool you can do?

Yes! Since we're really just riding on top of a decision-problem automaton, you can take a specification and invert it to produce an infinite stream of random strings that are all guaranteed to be parsed correctly.
If you're writing a language, this means _automatically generating all possible valid source files_.

And, you guessed it, this gets compiled down to Rust source as well, so your property-tests can be ridiculously effective.

## Acknowledgments

First off, to Haskell's parsing libraries (and UPenn's Haskell course) for showing me that parsers can even work this way. Most of the syntax is inspired by Haskell.

Second, to Rajeev Alur and Penn's CIS 262 for formally introducing me to nondeterministic finite automata.

Third, to Rust.
