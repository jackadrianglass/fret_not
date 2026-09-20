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

## Work log

`worklog/` holds one directory per slice of work actually tackled or presently in progress — `<idx>-<name>`, the index incrementing over time in the order work started. This is a record, not a plan: order reflects when something was picked up, not a sequence laid out in advance, and there's no dependency graph between entries. Each directory carries:

- `plan.md` — what we're trying to make (kept narrow in scope), a todo list of anticipated changes, a running Q&A log, and any external references. Written before implementation starts, revised as understanding shifts.
- `change-log.md` — one short, focused line per change, appended as work actually happens (not predicted in advance — that's what the plan's todo list is for).
- `conversation.md` — a narrative recap of how the plan evolved, written once the work is done.
- Anything else the work needs.

`todo.md` at the root holds ideas that haven't been started — the brainstormed backlog to pull from when picking up new work, and where anything deferred mid-project lands. Grouped loosely by theme for scanability; the grouping and its order carry no priority and no formal dependency. When an idea graduates into an active `worklog/` entry, drop it from `todo.md`.

## Starting a new piece of work

1. Check `todo.md` for a matching idea and pull its context if there's a fit.
2. Create `worklog/<idx>-current/` (next index; "current" is a placeholder — parallel in-flight entries just take different indices, so no collision). Don't settle on a real name yet.
3. Draft `plan.md`'s Plan and Changes sections from the request as understood so far, then pause for review.
4. Loop: append open questions to `plan.md`, prompt for review, fill in answers under each question as they come back, revise the plan — repeat until it's something worth executing.
5. Get an explicit go-ahead before implementing.
6. While implementing, append one-liners to `change-log.md` as changes land.
7. Once implementation is done, prompt for exploration and feedback without pre-explaining what to look at or for.
8. Rename the directory from `-current` to a real descriptive name once scope has actually settled (as soon as the plan stabilizes — no need to wait for the work to finish), and write `conversation.md` as a narrative recap once it's done.

Write `plan.md`, `change-log.md`, and `conversation.md` in one voice, as a single project record — not a transcript between two parties. `plan.md`'s Questions section is the deliberate exception, since it's structurally a Q&A exchange.

## Ask questions

I want to be in the loop on what's getting built, not find out after the fact. If a plan doesn't spell out a decision, ask instead of guessing and moving on — including small stuff. There's no dumb question here, and asking costs a lot less than me finding a wrong assumption baked in later.

Ask when:
- The decision affects how the app looks or feels — "pretty" is a judgment call, not a spec, and it's mine to make.
- A stated goal is ambiguous about what "done" actually looks like.
- You're about to make a call that one of `todo.md`'s open questions should have settled first.
- The obvious implementation and the one the plan implies aren't the same thing.

An answered question isn't a reason to stay quiet if it still doesn't sit right — decisions here aren't set in stone, and new context can reveal an old assumption was wrong. Revisit and ask.
