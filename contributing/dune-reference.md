# Dune reference

Quick notes on the build/package workflow this project uses. For anything
not covered here, go straight to the upstream docs linked in each section.

## What's pinned where

| File | Role |
|---|---|
| `dune-project` | Declares the package, its `(depends ...)` (including the OCaml version), dune language version |
| `dune.lock/` | Generated lockfile directory — exact resolved versions + source URLs for every dependency, checked into git, portable across platforms |
| `bin/dune`, `lib/dune`, `test/dune` | Per-directory build stanzas (executables/libraries and their `(libraries ...)`) |

This project uses **dune's own built-in package manager**, not a classic
`opam switch`. That's a newer (2025) dune feature, so most tutorials found
online describe `opam install <pkg>` / switches instead — that workflow
does not apply here.

## Common commands

| Command | What it does |
|---|---|
| `dune build` | Build everything; also resolves/fetches deps per `dune.lock/` |
| `dune exec bin/main.exe` (or `dune exec fret_not`) | Build and run the executable |
| `dune test` / `dune runtest` | Run the test suite (`test/`) |
| `dune build @fmt` | Show formatting diffs (via `ocamlformat`) |
| `dune build @fmt --auto-promote` | Apply formatting fixes in place |
| `dune build @doc` | Generate API docs via `odoc` → `_build/default/_doc/_html/index.html` |
| `dune pkg lock` | Re-solve dependencies and regenerate `dune.lock/` (run after editing `dune-project`'s `(depends ...)`) |

## Adding or updating a dependency

1. Add the package name to `(depends ...)` in `dune-project`.
2. Add it to the relevant `(libraries ...)` stanza in `bin/dune` or
   `lib/dune`.
3. Run `dune pkg lock` to re-solve and regenerate `dune.lock/`.
4. Commit the changed files under `dune.lock/` along with the
   `dune-project`/`dune` changes.

## Where to look things up

| Question | Resource |
|---|---|
| Dune stanza reference (`executable`, `library`, `rule`, ...) | [dune.readthedocs.io/en/stable/reference](https://dune.readthedocs.io/en/stable/reference/) |
| How dune's package management/lockfiles actually work | [Explanation: package management](https://dune.readthedocs.io/en/latest/explanation/package-management.html) |
| Tutorial: managing dependencies with dune pkg | [Managing Dependencies](https://dune.readthedocs.io/en/latest/tutorials/dune-package-management/dependencies.html) |
| Generating docs with odoc | [Generating Documentation](https://dune.readthedocs.io/en/latest/documentation.html) |
| Everything else / source | [github.com/ocaml/dune](https://github.com/ocaml/dune) |

## Generating docs

`dune build @doc` renders every `.mli` doc comment via `odoc` into
`_build/default/_doc/_html/index.html`. Requires `odoc` to be a resolvable
dependency (add it to `dune-project`'s `(depends ...)` under `{with-doc}`
if it isn't already pulled in transitively).
