# Fret Not

A guitar practice tool built around one idea: everything you practice should be
reusable when you compose, perform, or improvise — one system that carries you
from *practice* to *making music*, not six separate tools bolted together.

![Basic demo of current state](./demo.png)

Fully offline, fast to open and close mid-session, and pleasant to look at,
because it gets stared at a lot.

## The core idea

Most practice tools either show you a fixed shape on the fretboard or a fixed
tab of a song — both tied to specific frets and strings. Fret Not is built
around a different unit: a **chunk**, a movable pattern defined by scale
degree and rhythm relative to a key and mode, not by fixed positions.

Because a chunk is abstract, it can be:

- **Projected** onto the fretboard or into tab, in any key, at any position
  on the neck.
- **Extracted** back out of a tab you're learning, so a lick from a real song
  becomes a reusable, transposable chunk.
- **Varied** — re-permuted, sequenced, layered — and put in the context of a
  song's key, tempo, and chords, so practice material feels connected to
  actual music, not abstract drills.

That abstract-pattern ↔ concrete-fretboard bridge is the architectural core
of the project; every feature either builds it, feeds chunks into it, or
consumes chunks out of it.

## Status

Early and in active development. What exists today: the music-theory layer
(keys, modes, pentatonics, arpeggios), the fretboard projection layer
(degree ↔ position, notes-per-string shapes), the chunk model with a chord
progression and multi-fingering solver, and a fullscreen raylib UI with a
fretboard viewer and tab rendering. Nothing persists to disk yet — the chunk
library, tab editor, and variation engine are the next frontiers.

Orientation for working in the code is in [`AGENTS.md`](AGENTS.md).

## Build & run

Needs [devenv](https://devenv.sh) for the toolchain (OCaml 5.4.1, dune, and
the vendored raylib/raygui build deps).

```sh
devenv shell                     # enter the dev environment
dune build                       # build everything
dune exec bin/main.exe           # run the app
dune test                        # run the test suite
dune build @fmt --auto-promote   # apply formatting fixes
```
