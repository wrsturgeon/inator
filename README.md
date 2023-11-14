# `inator`: An evil parsing library.
## You supply the evil plan; we build the _**inator**_.

![Portrait of the eminent Dr. Heinz Doofenshmirtz](http://images6.fanpop.com/image/polls/1198000/1198459_1364687083851_full.jpg)

🚧 Development still ongoing! 🚧

## TL;DR

We ask for a specification and turn it into a graph that knows exactly what can happen at each step.
We then ruthlessly cut things that won't happen, combine identical ones, and output the result as a Rust file.
Compile this with the rest of your code and, _voila!_, you've got a hand-rolled zero-copy parser.

## The long explanation

Given a language like "`AAAAAAAAB` or `AAAAAAAAC`", we'd like to write a parser that takes the common-sense route:
blow through all the `A`s as a group, _then_ match on the `B` or `C` once.
This has several advantages:
- We never rewind the tape, so we don't need to store any previous input.
- In the limit, it's twice as fast as trying one then the other. With _n_ alternatives for the last character, it's _n_ times faster.
This general idea goes by the name _zero-copy streaming parsers_, and the pleasure of constructing them usually ranks similarly to being repeatedly stabbed.

Yet, as far as I know, no existing parsing library tries to optimize these cases at compile time, and for good reason: it's _hard_.

The problem is that, the more we can do with a parser, the less we can say about them before they run.
I've tried to strike a balance between the two with a model of computation similar to [pushdown automata](https://en.wikipedia.org/wiki/Pushdown_automaton) that maps nicely onto stack-based bare-metal code. Here's the definition, with overloaded terms _italicized_:
- A _parser_ is a _graph_.
- A _graph_ is a set of _states_ with an initial _index_.
- An _index_ is implementation-defined (literally, by a Rust trait), but it's usually either an natural number or a set thereof,
  corresponding to deterministic or nondeterministic evaluation, respectively.
- A _state_ is a set of _curried transitions_ and a (possibly empty) set of error messages; if the input stream ends on a state with no error messages, we accept the input.
- A _curried transition_ can take two forms:
    - Accept any input token and trigger one _transition_ for them all, or
    - Map a set of disjoint input ranges to potentially different transitions for each, and have an optional fallback transition if no ranges match.
  Note that we cannot inspect the stack before choosing a transition.
- A _transition_ is one of three things:
    - _Lateral_: Don't touch the stack; just move to a new state index.
    - _Return_: Pop from the stack (reject if the stack is empty) and move to the state index we just popped.
      Note that this is exactly how Rust's actual call stack works in assembly.
      Also note that we can't move to a _specific_ state with a return/pop statement; this would entail returning a function pointer at runtime, which is pointlessly (get it?) slow.
    - _Call_: Push a specified _destination index_ onto the stack and move to a specified _detour index_.
      Note that the detour index needs to have a return/pop statement, at which it will move to the specified destination index,
      but (as of yet) we don't check that a return statement is reachable from any given detour index.
You may have noticed that the structure of states and transitions maps remarkably well onto functions and calls:
- States are functions;
- Lateral transitions are [tail calls](https://en.wikipedia.org/wiki/Tail_call);
- Return transitions are actual `return` statements; and
- Calls are function calls that are not at the end of a block of execution (if they were, they would be lateral transitions).
And this is exactly how we [~~trans~~compile](https://hisham.hm/2021/02/25/compiler-versus-transpiler-what-is-a-compiler-anyway/) it.

Lastly, on top of this graph, parsers _output data_. We can't prove anything about the values you compute along the way—it's essentially having one model of computation (Rust) riding another (automata)—but, in practice, being able to have your parsers _output_ something (e.g., an abstract syntax tree) without having to run a lexer first is invaluable.

The [`automata` directory](automata/) in this repository contains both an interpreter and a compiler for this language, and I remain extremely confident that their operation is always equivalent, but property-testing involving compiling Rust code is extremely difficult. In the future, I would like to prove their equivalence in [Coq](https://github.com/coq/coq) and [extract](https://softwarefoundations.cis.upenn.edu/lf-current/Extraction.html) the proof into Haskell and OCaml versions of this library. Way in the future! 🔮

## What does this whole process look like?

Surprisingly, it looks a lot like just writing down what you want.
The key idea is that ***parsers are data***, and you can pass them around, modify them, and combine them just like anything else.

Here's how we parse either "abc" or "azc":
```rust
use inator::toss; // Combinator that filters for a certain character, then forgets it.
let a = toss('a');
let b = toss('b');
let c = toss('c');
let z = toss('z');
let abc_azc = a >> (b | z) >> c;
```
The above gets compiled down a function that takes an iterator,
- checks that the first item is `a`;
- runs a `match` statement on the second item without rewinding, sending both `b` and `z` to the same third state;
- checks that the third item is `c`; then
- checks that the next item is `None`.
It's not remarkable when spelled out, but most simple parsers would allocate a buffer, read input into the buffer, rewind to the top, try `abc`, rewind, try `abz`, then reject.
By the time a similar parser has even allocated its buffer, let alone read the input, our parser is almost surely already done.
Plus, this approach to parsing requires zero allocations, so even microcontrollers running bare-metal code couldn't get any faster if you tried.

Then we can take that whole above parser and pass it around, e.g. to put it in parentheses:
```rust
// Copied from above:
use inator::toss;
let a = toss('a');
let b = toss('b');
let c = toss('c');
let z = toss('z');
let abc_azc = a >> (b | z) >> c;

// Function from a parser to another parser!
let parenthesized = |p| toss('(') >> p >> toss(')');

let paren_abc_azc = parenthesized(
    abc_azc, // <-- Parser we just made above, passed around as data
);
```
Above, if `p` is a parser that accepts `ABC`, then `parenthesized(p)` will accept `(ABC)`, and so on for any language other than `ABC`. Simple as that.

If you need to _nest_ parentheses (or any other delimiters) and verify that everything matches up, there's a built-in function for that. `region` takes five arguments:
- `name`, a `&'static string` describing the region (e.g. in error messages);
- `open`, a parser that opens the region (here, it would be `toss('(')`);
- `contents`, the parser that runs inside the region;
- `close`, a parser that closes the region (here, it would be `toss(')')`); and
- `combine`, which is a bit more complicated.
    - Every parser returns a value, but after a call, we have two: what we had before, and the return value from the call.
      You can combine these two values in any way you'd like, including by throwing one or the other out.

## Anything else cool you can do?

Yes! Since we're really just riding on top of a decision-problem automaton, I'm working on (but confident about) taking a specification and inverting it to fuzz with an infinite stream of strings that are all guaranteed to be parsed correctly.
If you're writing a language, this means _automatically generating all possible valid source files_, which would be huge. After the redesign of automata above, this is back in the to-do pile.

## Why not other parsing libraries?

Please try other parsing libraries! My primary goal was a personal tool, but it turned out much better than I expected, so please put it to good use and send feedback!

## Acknowledgments

Haskell's parsing libraries (and UPenn's Haskell course), for showing me that parsers can even work this way.

Rajeev Alur (and UPenn's CIS 262), for formally introducing me to nondeterministic finite automata.

Rust, for making this possible.
