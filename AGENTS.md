# Working notes for this project

## Project documentation

`docs/` holds documentation of the project's own domain model, kept in sync with `lib/` as that model evolves rather than describing an earlier version of it:

- [`docs/chunk-and-fretboard-model.md`](docs/chunk-and-fretboard-model.md) — how the chunk idea maps to Key/Mode/Tuning/Fretboard, from the abstract side to the concrete one and back.

## Contributing references

`contributing/` holds one reference file per technology this project depends on — OCaml/Base, Dune, devenv, raylib/raygui. Each is a cheat sheet (versions, common commands, where things live in this repo) plus links to the upstream docs, not a tutorial. Check the relevant file before reaching for general knowledge or a web search — this project pins specific versions and workflows (e.g. `base` instead of `Stdlib`, dune's own package manager instead of opam switches) that differ from what most generic OCaml guidance assumes.

- [`contributing/ocaml-reference.md`](contributing/ocaml-reference.md)
- [`contributing/dune-reference.md`](contributing/dune-reference.md)
- [`contributing/devenv-reference.md`](contributing/devenv-reference.md)
- [`contributing/raylib-raygui-reference.md`](contributing/raylib-raygui-reference.md)

Assume the shell you're already running in is a `devenv shell` — `dune`, `ocaml`, `opam`, etc. are already on `PATH`. Don't run `devenv shell` yourself.

`contributing/` also holds how code in this project should be written:

- [`contributing/coding-guidelines.md`](contributing/coding-guidelines.md) — no comments, top-down flow built from bottom-up pieces, functional-first with computation kept separate from the raylib/raygui shell, testing the computations rather than the shell, and general craftsmanship. Read it before writing or reviewing code, not just when something looks off.

## Ask questions

I want to be in the loop on what's getting built, not find out after the fact. If a plan doesn't spell out a decision, ask instead of guessing and moving on — including small stuff. There's no dumb question here, and asking costs a lot less than me finding a wrong assumption baked in later.

Ask when:
- The decision affects how the app looks or feels — "pretty" is a judgment call, not a spec, and it's mine to make.
- A stated goal is ambiguous about what "done" actually looks like.
- You're about to make a call that one of `todo.md`'s open questions should have settled first.
- The obvious implementation and the one the plan implies aren't the same thing.

An answered question isn't a reason to stay quiet if it still doesn't sit right — decisions here aren't set in stone, and new context can reveal an old assumption was wrong. Revisit and ask.
