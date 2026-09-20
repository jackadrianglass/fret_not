# Coding guidelines

How we write OCaml in this project. These are principles, not a lint
config — use judgment, but if you're about to break one, that's worth a
second look (and worth saying so in the task file if the reason isn't
obvious).

## 1. No comments

Comments are almost always evidence the code didn't say what it meant.
Prefer:

- Names that state intent (`fret_for_degree`, not `f`; `is_open_string`,
  not `check`)
- Types that make illegal states unrepresentable, so the type itself is
  the documentation
- Small functions with a single, nameable responsibility

The one exception: a comment that captures a *why* a reader couldn't get
from the code — a subtle invariant, a workaround for a specific upstream
bug, a non-obvious reason a "better" approach doesn't work here. If
deleting the comment wouldn't leave a future reader confused, don't write
it.

This applies to `.mli` doc comments too. A signature's types and names are
the primary documentation; write a `(** ... *)` doc comment only for the
rare function whose contract genuinely isn't clear from its type alone —
not as a matter of course. `dune build @doc` (see
[`dune-reference.md`](dune-reference.md#generating-docs)) will mostly
render bare signatures under this policy, and that's expected.

## 2. Top-down flow, bottom-up building blocks

Write the entry point / orchestrating function first, and make it read
like a summary of what happens — a sequence of well-named calls to things
defined below it, not the mechanics inlined. Someone reading only that top
function should understand the shape of what's happening without opening
another file.

Build the pieces it calls bottom-up: small, separable functions or modules
for genuinely distinct ideas (parsing a tuning, projecting a degree onto a
fret, computing an interval) that don't know about each other's callers.
If two of them start blurring together, that's a signal to either merge
them (they were never separate ideas) or find the real boundary (they
were separate, but drawn in the wrong place).

## 3. Functional first

Default to:

- Pure functions — same input, same output, no hidden state
- Modeling the domain with types before writing logic (a `Degree.t`,
  `Tuning.t` you can't misuse, rather than raw ints/strings threaded
  around)
- Separating computation from action — the raylib/raygui shell (window,
  input, drawing) is the action layer; the music-theory/tab logic
  underneath should stay pure and callable with no window open

The program's top-level shape already does this: state is threaded
through the render loop, not mutated (see
[`raylib-raygui-reference.md`](raylib-raygui-reference.md#program-shape)).
Carry the same discipline into the domain logic — compute the *what*, let
a thin shell handle drawing it.

Reach for something else (a `ref`, a loop with mutation, an exception for
control flow) when the pure/immutable version is genuinely worse —
noisier, meaningfully slower, or fighting raylib's own imperative shape —
not by default.

## 4. Test the computations, not the shell

Pure domain logic (key/mode math, tuning resolution, degree↔fret
projection, permutation/variation logic) should be cheap and fast to test
— no window, no raylib init, no I/O. That's the payoff of keeping it pure:
`dune test` should run in well under a second.

Tests run through [Alcotest](https://github.com/mirage/alcotest)
(`test/dune`,
`(libraries fret_not alcotest)`) — grouped test cases via
`Alcotest.test_case` and `Alcotest.run`, checked with `Alcotest.check`'s
typed combinators. Still don't add a test just because the framework makes
it easy to; a test earns its place by catching something a type error or a
five-second manual check wouldn't.

Don't test the raylib/raygui shell itself — an immediate-mode GUI's
contract *is* correctly drawing pixels every frame, which isn't something
worth asserting on. If a bug lives there, it's a run-the-app-and-look bug,
not a unit test.

If a test doesn't tell you something a type error or a five-second manual
check wouldn't already tell you, don't add it — and delete tests that stop
earning their keep.

## 5. Make it look like you care

Run `dune build @fmt --auto-promote` before calling something done —
formatting isn't a place to spend judgment. No dead code, no
commented-out blocks, no `TODO`s without a task file backing them (the
TODO goes in `tasks/`, per the root [`CLAUDE.md`](../CLAUDE.md)). Warnings
aren't noise — an unused variable or a nonexhaustive match is dune telling
you something before a person has to. If it doesn't build clean, it isn't
done.
