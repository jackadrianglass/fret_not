# OCaml / Base reference

Quick notes on the language setup this project uses. For anything not
covered here, go straight to the upstream docs linked in each section.

## Version

| | |
|---|---|
| OCaml | `5.4.1`, pinned in `dune-project` |
| Standard library | [`base`](https://github.com/janestreet/base) `v0.17.3` (Jane Street), not the OCaml `Stdlib` |
| Test framework | [`alcotest`](https://github.com/mirage/alcotest) `1.9.1`, `:with-test` only — see [`coding-guidelines.md`](coding-guidelines.md#4-test-the-computations-not-the-shell) for how it's used |

Both are pinned via dune's own lockfile (`dune.lock/`) — see
[`dune-reference.md`](dune-reference.md) for how that resolution works.

## Why Base instead of Stdlib

This project uses `base` as its standard library. `base` is a drop-in
replacement for large parts of `Stdlib` with different defaults:

- No polymorphic `compare`/`equal`/`(=)` on structural types — every
  comparable type exports its own `compare`/`equal`, and `Poly.compare` /
  `Poly.equal` exist for the rare case you actually want the structural
  version.
- Functions that take a function argument use labeled arguments, e.g.
  `List.map ~f:(fun x -> ...)`, `List.filter ~f:...`.
- No `open Stdlib` — files that use `base` open `Base` instead
  (`open! Base` if you also want unused-open warnings suppressed for it).
- `Option`, `Result`, `List`, `Array`, `String`, `Map`, `Set`, etc. all have
  `Base`-flavored signatures — check `Base.X` before reaching for `Stdlib.X`.

To wire it into a library or executable's `dune` file: `(libraries base)`
(already set on `lib/dune`).

## Where to look things up

| Question | Resource |
|---|---|
| Core language syntax/semantics | [OCaml manual](https://ocaml.org/manual/) |
| Official style/naming guidelines | [ocaml.org/docs/guidelines](https://ocaml.org/docs/guidelines) |
| `Base` module API (this project's stdlib) | [Base API docs](https://ocaml.org/p/base/v0.17.3/doc/Base/index.html) |
| `Base` source / design rationale | [github.com/janestreet/base](https://github.com/janestreet/base) |
| Idiomatic examples written against `Base` | [*Real World OCaml*](https://dev.realworldocaml.org/) — uses `Base` throughout, unlike most other OCaml tutorials which assume `Stdlib` |
| Package/API search across all of opam | [ocaml.org/packages](https://ocaml.org/packages) and [v3.ocaml.org search](https://v3.ocaml.org/search) |

Note: most OCaml tutorials/Stack Overflow answers found online assume
`Stdlib`, not `Base` — argument order and labeling will often be different
from what you find there. When in doubt, check the `Base` API docs above
before trusting a generic OCaml snippet.

## Docs for this project's own code

Interfaces (`.mli`) and their doc comments are what `odoc` renders — see
[`dune-reference.md`](dune-reference.md#generating-docs) for the build
command.
