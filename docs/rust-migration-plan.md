# Rust migration plan

Goal: port everything that is not GUI code to Rust, keeping behavior identical,
then use the resulting core as the stable base for experimenting with Rust GUI
toolkits. The port is complete and the OCaml tree (implementation, tests, and
build system) has been removed; the Rust core is the sole implementation and
this document now serves as the record of what was captured for the GUI work.

## Scope split

Ported (core logic, no rendering knowledge):

- `crates/absolute/src` — pitch_class, alteration, letter, spelled_pitch, note
- `crates/relative/src` — scale_degree, mode, key, scales, degree_reference,
  chunk
- `crates/fretboard/src` — fretboard_position, tuning, instrument, fretboard,
  shape, reach, chunk_solver

Not ported, and now gone with the OCaml tree (GUI logic, captured behaviorally
below):

- `lib/layout` — pure geometry, but existed only to serve the views
- `lib/view` — UI state and presentation constants
- `bin/` — raylib/raygui rendering and the frame loop

The layout/view modules are cheap to reimplement inside whichever toolkit wins
the experiment. The important part is that the *feature capture* below (what
the app lets the user do) survives the rewrite.

## Target shape

A Cargo workspace where the OCaml layer convention becomes compiler-enforced
dependency edges (the one structural improvement this port buys for free):

```
crates/
  absolute/     pitch_class, alteration, letter, spelled_pitch, note
  relative/     scale_degree, mode, key, scales, degree_reference, chunk
  fretboard/    fretboard_position, tuning, instrument, fretboard, shape,
                reach, chunk_solver
```

- `relative` depends on `absolute`; `fretboard` depends on both. Upward imports
  no longer compile, where before they were only a convention.
- Later: a `gui/` member (or several spike crates, one per toolkit candidate)
  depending on the core crates.

Zero external dependencies for the core crates — the OCaml core only uses
`base`, and none of it needs anything beyond std.

## Porting conventions

- Type-for-type translation: OCaml variants become Rust enums, records become
  structs. Small value types (`PitchClass`, `Letter`, `Alteration`,
  `SpelledPitch`, `Note`, `FretboardPosition`, `Reach`) can be `Copy`.
- `ppx_deriving eq/ord` becomes `#[derive(PartialEq, Eq, PartialOrd, Ord)]` —
  except the two deliberate hand-writes, which stay hand-written and carry the
  same comments:
  - `Alteration`: ordered by semitones, not declaration order.
  - `Note`: ordered by sounding semitone, spelling breaks ties.
- OCaml `option` becomes `Option`; `*_exn` functions become panics with the
  invariant documented on the function, or a `Result` if callers can reasonably
  hit the bad case. `Scale_degree.spelled` documents "raises when the spelling
  would need more than a double sharp/flat" — keep that as a panic; callers
  inside the project only spell diatonic degrees.
- Keep functions pure and module-scoped; no traits or generics that the OCaml
  code doesn't already imply.
- No comments beyond the invariant "why" comments already present (the
  tonic-anchored octave in `degree_reference`, the resolve-then-choose-string
  invariant in `chunk_solver` — those comments must move with the code).
- Tests live inline per module (`#[cfg(test)]`), mirroring the alcotest suites
  one-for-one with the same cases and expected values. The existing test files
  are the port's acceptance spec.

## Phases

Status: the core port is complete. `crates/{absolute,relative,fretboard}`
exist with the full module set and test suites; the differential check
(phase 5) passes byte-for-byte. What remains is the GUI experimentation.

1. Workspace scaffold: `crates/{absolute,relative,fretboard}`, empty modules,
   `cargo test` green. (Done; the Rust toolchain is already in `devenv.nix`
   via `languages.rust`.)
2. `absolute` crate: five modules plus their test suites (the OCaml suites in
   `test/absolute/`, ~9 KB of cases). (Done: 19 tests.)
3. `relative` crate: six modules plus suites. The subtle one is
   `degree_reference.interval` — octave is tonic-anchored, degree 1 octave 0 is
   the tonic — and `chunk.reframe`'s octave carry on degree wrap.
   (Done: 25 tests.)
4. `fretboard` crate: seven modules plus suites. `chunk_solver.positions` must
   resolve each note's absolute semitone first and then choose only which
   string; the regression suite in `chunk_solver_test.ml` exists specifically to
   keep non-tonic chord fingerings correct. `shape.positions` drops shapes to
   their lowest playable octave and omits unplayable ones. (Done: 40 tests.)
5. Differential check before calling the core done: a tiny OCaml program (and a
   matching Rust one) that prints, e.g., `Chunk_solver.positions` output for a
   set of chunks across keys/tunings, plus `Shape.positions` for the three
   presets. Same output byte-for-byte. This catches the class of bug the
   chunk_solver regression tests were written for, on inputs the unit tests
   don't cover. (Done: the two dumps were byte-identical over 2307 lines
   covering 4 keys x 4 tunings, solver output for 7 note sets each, all shape
   presets, and degree windows. The OCaml counterpart was removed with the
   OCaml tree; the Rust dumper survives as
   `crates/fretboard/src/bin/dump_fixtures.rs` — re-baseline it deliberately
   if the core's output semantics ever change on purpose.)

Estimated size: 18 modules, all small; the OCaml core plus tests is roughly
60 KB of source. Phases 2–4 are each a sitting's work.

---

# Current GUI feature set (for toolkit experiments)

What the app did at the time of the migration, described toolkit-agnostically
so it can be rebuilt against any Rust GUI toolkit. The raygui-specific
implementation quirks (dropdown toggle pulses, style-propagation traps) were
workarounds, not features, and went unrecorded — do not re-create them.

## App shell

- Single window, target-FPS immediate-mode loop: state is immutable and
  threaded through the frame; each frame recomputes geometry (resize-safe).
- Resizable window, starts maximized, with a minimum size derived from content
  layout. Width fills the screen; content is anchored by rows.
- Vertical stack, top to bottom:
  1. Control bar (fixed height, tinted background, rule along its bottom edge)
  2. Page content: chunk editor page or tab page (whichever the page toggle
     selects)
  3. Fretboard view, pinned to the window bottom regardless of page

## Control bar

- Left side: page toggle — "Edit" / "View" (chunk editing / tab viewing).
- Right side, flush to the right edge: tonic dropdown (12 spelled tonic names),
  key quality dropdown (Major / Minor), static "showing" label, label-mode
  dropdown (Degrees / Notes).
- Dropdowns open and close on click; open state is part of the UI state.
- Tonic + quality together determine the key and mode used by everything below.

## Fretboard view (always visible, bottom)

- Grid: one horizontal line per string, light vertical line per fret, fret
  numbers drawn below the lowest string. Fret count comes from the instrument
  (max fret per string), not from any UI constant.
- Every playable position on the instrument gets a dot:
  - In-key positions: filled dot in the theme accent color, degree-1 positions
    additionally ringed with a black halo, label centered in the dot showing
    either the degree number or the note name (per label-mode dropdown).
  - Off-key positions: hollow light-gray circle, no label.

## Chunk editing page

- A chunk is an equal-duration slot list; each slot is a rest or a
  degree-reference note (degree 1–7, alteration b/n/#, octave 0 or 1). Slot
  presence is the rhythm.
- Slot strip: one cell per slot, label is the degree label ("1", "b3", with a
  trailing apostrophe per octave above the tonic) or "." for a rest; rest cells
  render dimmed, note cells in the accent color; the selected slot has a
  cursor outline. Click a cell to select it.
- "+" button after the last cell appends a rest (bounded by a max-slot count);
  "-" removes the selected slot (cursor moves left, or to 0 at the left edge).
- Inspector row below the strip: Note/Rest toggle, degree dropdown (1–7),
  alteration toggle (b/n/#), octave toggle (0/1). Editing a slot that is a rest
  converts it into a note holding the edited value at its defaults.

## Tab viewing page

- Tab rows: one line per string, labeled at the left with the string's absolute
  open note in SPN (so E2 vs E4 are distinct), note columns showing fret
  numbers, and a thick section-divider rule below.
- Currently a placeholder: an empty grid, no notes. The intended data source
  (selected chunk fingering from the solver) is not wired yet.

## Known not-yet-features (so experiments can aim at them)

- Chunk solver is library-only: no UI flow for "pick a fingering of the edited
  chunk and see it on the fretboard/tab".
- Reach ranking exists (`Reach.semitone_span`) but no pick-and-practice flow.
- No persistence of any state.
- Tab page has no data source.
