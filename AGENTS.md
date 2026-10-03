# Codebase map

Orientation for agents (and humans) working in this repo: where things live,
how they connect, and the conventions to follow. The project's purpose is
in `readme.md`; this file is the code-level counterpart.

## The one-paragraph mental model

The whole project is a bridge between **abstract, degree-relative music** and
**concrete string/fret positions**. Music theory (`lib/theory`) defines what a
key/mode/scale-degree *is*. A chunk (`lib/chunk`) is a movable sequence of
those degrees. `lib/fretboard` projects degrees onto actual positions for a
given tuning (and extracts them back). `Chunk_solver` goes further and finds
every playable fingering for a chunk. The `bin/` + `lib/view` + `lib/layout`
layer renders it all with raylib/raygui. Everything else feeds that bridge or
consumes it.

## Commands

```sh
devenv shell                     # enter the toolchain (OCaml 5.4.1, dune)
dune build                       # build
dune exec bin/main.exe            # run the app (borderless fullscreen)
dune test                        # alcotest suite (test/test_fret_not.ml wires it)
dune build @fmt --auto-promote   # ocamlformat
```

Build system: dune 3.23, `fret_not.opam` generated from `dune-project`
(edit `dune-project`, not the opam file). Deps are deliberately tiny:
`base`, `raylib`, `raygui`, `alcotest` (test only), `ocamlformat`.

## Module map

```
lib/theory/        what the music is (no fretboard knowledge)
  pitch_class.ml   int mod 12 + names; all arithmetic goes through add/of_int
  alteration.ml    Natural | Sharp | Flat | Double_sharp | Double_flat — the
                   notational accidental of a scale degree
  letter.ml        the 7 natural letters; only these have natural pitch classes
  spelled_pitch.ml a letter + an alteration — SPN minus the octave (F#, Bb);
                   semitone is NOT folded into an octave (Cb = -1, B# = 12),
                   which is what lets Note handle the B/C octave wrap
  note.ml          a spelled pitch + an octave — a full SPN note (C4, E2, Bb3);
                   semitone = 12*(octave+1) + spelled semitone (C4 = 60, A4 = 69);
                   pitch class is derived, never stored
  scale_degree.ml  {degree: 1..7; alteration} — a notated scale degree (1, b3,
                   #4). Pitch class is *derived* (pitch_class ~root ~mode),
                   never stored; the 7 degrees of any mode are the same
                   notational objects (Scale_degree.diatonic). spelled ~root
                   turns a degree into an SPN spelling: letter = root letter +
                   (degree - 1), alteration field is the accidental
  mode.ml          7 diatonic modes. Key insight: a mode is a *rotation* of the
                   major scale — root_offset_semitones indexes into
                   [0;2;4;5;7;9;11], pitch_classes() rotates the parent scale
  key.ml            {tonic: Spelled_pitch.t; Major|Minor}; mode_root() lifts
                   tonic to any mode via the parent-major trick and returns a
                   *spelled* pitch; mode_pitch_classes() is the scale as pitch
                   classes; mode_of_quality() maps Major/Minor to Ionian/Aeolian
  pentatonic.ml    degree-subset presets (major/minor pentatonic)
  arpeggio.ml      degree-subset presets (triads)
  degree_reference.ml      {scale_degree; octave} — THE abstract note, and the
                   chunk layer's domain type. Octave is TONIC-anchored: octave 0
                   spans from the tonic up to the octave above, so degree 1
                   octave 0 IS the tonic. interval() is THE resolution primitive
                   (semitones above the tonic); note() builds an absolute SPN
                   note on top of it.

lib/fretboard/     the abstract↔concrete bridge
  fretboard_position.ml   {string_index; fret} — a concrete position
  tuning.ml        open strings as absolute SPN notes (standard = E2 A2 D3 G3
                   B3 E4), low to high. Absolute pitch is the point: pitch
                   classes cannot distinguish E2 from E3 (Nashville tuning,
                   bass, octave-displaced tunings). retune_string shifts one
                   string and spells minimally; presets derive from standard.
  instrument.ml    tuning + fret count *per string*. Everything that asks
                   "can this be played here" (playable, range) goes through an
                   instrument; semitone_at resolves a position to sounding
                   pitch. This is where fret availability lives — not in view
                   config.
  fretboard.ml     the slim bridge: positions_of_semitone (the one position
                   enumerator, shared with the solver), degree_positions
                   (degree → positions, instrument-clamped), degree_at
                   (position → degree option, None off-key),
                   degrees_in_window (every in-key playable position WITH its
                   degree). root_note is the octave register all resolution
                   here is anchored to; degree_at round-trips through
                   Degree_reference.interval.
  reach.ml         finger-stretch geometry: fret span, string span, and the
                   sounding interval span between two positions (tuning-aware —
                   cross-string stretch depends on the tuning's intervals).
                   Only between() is wired so far (Chunk_solver's distance);
                   ranking by semitone_span awaits the pick-and-practice flow.
  shape.ml         notes-per-string shapes over ANY degree list — the diatonic
                   degrees give the seven 3NPS positions, Pentatonic's the five
                   2NPS, Arpeggio's the three 1NPS inversions. Shapes drop to
                   their lowest playable octave; unplayable shapes are omitted.

lib/chunk/         the movable pattern layer (newest, most conceptually central)
  slot.ml          Rest | Note of Degree_reference — slot presence IS the
                   rhythm; all slots equal duration (deliberate simplification)
  chunk.ml         Slot.t list; reframe ~root_degree rotates degrees mod 7 with
                   octave carry — this is what makes chunks transposable
  chord_progression.ml     int list of scale-degree roots (e.g. [1;4;5]);
                   apply_to_chunk repeats a chunk per chord via reframe
  chunk_solver.ml  positions() — every fingering of a reframed chunk, ranked by
                   reach distance from a start anchor. CRITICAL invariant:
                   resolve each note's absolute semitone (via
                   Fretboard.degree_note) FIRST, then choose only *which
                   string*. Skipping this broke every non-tonic chord (see
                   chunk_solver_test.ml regression tests). The first note's
                   octave window is derived from Instrument.range — no magic
                   sweep constants — and candidate enumeration reuses
                   Fretboard.positions_of_semitone.

lib/layout/        pure geometry, no raylib, fully unit-tested
  fretboard_layout.ml      string/fret x,y math for the fretboard view
  tab_layout.ml           tab row geometry (canvas_height, string_y, note_x)
  row_layout.ml           horizontal control-row math (x_positions,
                          dropdown_width)

lib/view/          renderable state/config (no raylib calls; testable)
  fretboard_view_config.ml  all presentation constants (window size, fonts,
                            radii, spacing) plus the Instrument (tuning +
                            frets per string). The Instrument is the only
                            source of fret truth: the drawn window is
                            Instrument.max_fret.
  tab_view_config.ml        tab presentation constants
  fretboard_view_state.ml   the UI state: dropdown indices (tonic, quality,
                            scale, position, label mode). The scale dropdown
                            picks a DEGREE LIST (Diatonic/Pentatonic/Arpeggio
                            via the theory presets) which drives highlighting
                            and Shape.positions (3/2/1 notes per string).
                            Tonic dropdown indexes the tonics list (spelled,
                            12 entries); position_options is derived from the
                            actual shape list so it never advertises a shape
                            the instrument can't play

bin/               raylib/raygui rendering + the frame loop
  main.ml          frame loop: compute layout per frame (resize!), tab view on
                   top, fretboard below, controls return next state. Window is
                   borderless-fullscreen, width fills screen, content centered
  fretboard_view.ml  draws grid, position dots (in-key filled / off-key hollow,
                     root halo, dimming when a position filter is active), and
                     the dropdown control bar via raygui
  tab_view.ml      draws tab strings (labeled with absolute SPN open notes:
                     E2 vs E4) + fret numbers; selected_position notes render
                     as tab; returns the y where the fretboard starts
```

## Data flow (one frame)

`Fretboard_view_state.t` (indices) → `key`/`mode` →
`Fretboard.degrees_in_window` / `Shape.positions` (concrete positions, clamped
to the config's Instrument) → `Tab_view.draw` (prints them as tab) +
`Fretboard_view.draw_fret_positions` (dots) → `draw_controls` returns next `t`.
Pure derivation, immutable state, state transitions only through the
control-drawing pass.

Chunk path (library code only, no UI yet): `Chunk.reframe` (harmonic
resolution per chord) → `Chunk_solver.positions` (every fingering, sorted by
distance) → whoever calls it (bin/main.ml doesn't yet; todo: pick-and-practice
flow).

## Conventions that will bite you if ignored

House style:

- **No comments** unless the code can't say it (a *why* an reader can't get
  otherwise). Existing comments of that kind exist in `bin/fretboard_view.ml`
  and `bin/main.ml` (raygui quirks, resize timing) — match that bar.
- Top-down: the orchestrating function reads like a summary; build the pieces
  bottom-up.
- Functional first; modules don't know about each other's callers.
- `open! Base` at the top of every file; layout/view modules take
  Float config and convert to pixels at the raylib boundary
  (`to_pixels`/`shift` in bin).
- Layout code is float math in `lib/layout`, tested without raylib; raylib
  calls stay in `bin/`. `lib/view` holds state/config, also raylib-free —
  keep that separation.
- ocamlformat profile is in `.ocamlformat` (double-semicolon let bindings,
  sparse type decls) — run `@fmt --auto-promote` before calling something
  done. Warnings aren't noise; if it doesn't build clean, it isn't done.
- Tests mirror the lib directory (`test/theory/`, `test/chunk/`, ...); every
  module gets an alcotest suite wired into `test/test_fret_not.ml`.
- Test the computations, not the shell: no window, no raylib init in tests.
  Don't add a test a type error or five-second manual check wouldn't catch.
- No persistence anywhere yet — a deliberate deferral, not an oversight.

## Toolchain & environment notes

(The `contributing/` reference docs were removed; these are the facts from
them that aren't recoverable from the code itself.)

- **Base, not Stdlib.** `base` v0.17 is the stdlib here: labeled function
  args (`List.map ~f:...`), no polymorphic compare/equal (each type exports
  its own; `Poly.equal` for the rare structural case), `open! Base` per
  file. Most OCaml snippets online assume `Stdlib` — check the Base API docs
  before trusting them.
- **dune's built-in package manager, not opam switches.** Dependency
  versions (OCaml 5.4.1, raylib/raygui 2.2.2, base, alcotest) are pinned in
  the committed `dune.lock/`, not by an opam switch. To change a dep: edit
  `(depends ...)` in `dune-project` + the `(libraries ...)` stanza, then
  `dune pkg lock`, and commit the `dune.lock/` changes. `fret_not.opam` is
  generated — edit `dune-project` instead.
- **devenv provides only the toolchain** (ocaml, dune, opam, LSP; `libffi`
  is there because the raylib bindings use ctypes). `devenv shell` is
  manual — no direnv auto-entry.
- **raylib/raygui bindings** are from github.com/tjammer/raylib-ocaml with
  the C sources vendored, so no system raylib install is needed. Depending
  on `raygui` re-exports `Raylib`.
- **raygui quirks (already worked around in `bin/`):**
  - `dropdown_box` returns a one-frame toggle pulse, not the new edit-mode
    value — the caller must XOR it against threaded open-state, or the box
    flashes open for one frame and snaps shut.
  - In raygui 2.2.2, `dropdown_box` positions its text via `Default`'s
    `Text_padding`, not `DropdownBox`'s — styling `DropdownBox`'s padding is
    silently ignored (see `Fretboard_view.setup`).
  - Widgets are immediate-mode: state threaded through the recursive
    render loop, never mutated in place.
