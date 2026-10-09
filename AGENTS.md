# Codebase map

This document outlines how to navigate the code base

Note that the following files aren't targetting you and are for the users only. Reading
them will likely misdirect you and cause confusion for everyone.
- `readme.md` meant to give context for other contributors that isn't relevant to the development
  of the project
- `backlog.md` meant as a scratch pad for the humans sorting out their ideas

Again, do not read these files (unless explicitely directed by the user). They will never be helpful
and you should ask the user instead for direction.

## Condensed mental model

The primary boundaries are
- Absolute notes layer (`crates/absolute/src`) define what a sounding note is (SPN: C4, E2, Bb3)
- Relative music layer (`crates/relative/src`) defines what a scale degree, key, and chunk are
    — a chunk is a movable sequence of degree-relative notes, and resolves degrees
      against a key into absolute notes.
- Fretboard layer (`crates/fretboard/src`) maps those onto concrete string/fret positions
  for a given instrument, including finding every playable fingering of a chunk.
- The GUI layer is not built yet. `docs/rust-migration-plan.md` records the feature set of
  the previous (removed) OCaml/raylib GUI so the Rust toolkit experiments can aim at it.

Dependencies only point downward, enforced by the cargo workspace layout: `relative`
depends on `absolute`, `fretboard` depends on both, and nothing below imports upward.
Future GUI crates sit beside `fretboard` and may import everything.

## Toolchain

The tools available are
- rust 1.97 via cargo (devenv `languages.rust`); Primary development language
- nushell 0.115.1; Shell for scripting

No external crate dependencies — the core is std-only by design.

## Commands

Assume that you are operating in the devenv shell and that all the tools are
available to you.

```sh
cargo test --workspace    # the test suite (all tests inline per module)
cargo fmt                 # rustfmt; run before calling something done
cargo build --workspace   # build
```

Building, checking, formatting, and testing are allowlisted — run them freely.
`cargo run` is deliberately NOT allowlisted: ask the human to run the
application and give feedback whenever you have something working. Keep them
actively involved in the process. UI work especially goes through them —
propose the change before implementing it, and let them judge the result.

Whenever you encounter things that you wish you knew when you started the task,
add/update a reference document in `notes/` for that library (recreate the
directory when needed). Don't complain about how you arrived at the conclusions.
Just write yourself notes to be kind to your future self.

## Module map

```
crates/absolute/src/       what the note IS — SPN vocabulary, imports nothing internal
  pitch_class.rs           newtype over int mod 12 + chromatic names; all arithmetic
                           goes through of_int/add (rem_euclid, so negatives wrap)
  alteration.rs            Natural | Sharp | Flat | DoubleSharp | DoubleFlat. Ord is
                           hand-written by semitones, NOT declaration order — say why
                           at the definition (it already does)
  letter.rs                the 7 natural letters; only these have natural pitch classes.
                           offset() wraps the letter cycle (F + 6 -> D)
  spelled_pitch.rs         a letter + an alteration — SPN minus the octave (F#, Bb);
                           semitone() is NOT folded into an octave (Cb = -1, B# = 12),
                           which is what lets Note handle the B/C octave wrap.
                           alteration_for() picks the smallest representable accidental
  note.rs                  a spelled pitch + an octave — a full SPN note (C4, E2, Bb3);
                           semitone() = 12*(octave+1) + spelled semitone (C4 = 60,
                           A4 = 69); pitch class is derived, never stored. Ord is
                           hand-written: sounding semitone first, spelling breaks
                           ties — say why at the definition

crates/relative/src/       what the degree IS — degree-relative music, no fretboard
                           knowledge; resolves against a key into absolute notes
  scale_degree.rs          {degree: 1..7, alteration} — a notated scale degree (1, b3,
                           #4). pitch_class() is derived (root + mode), never stored;
                           the 7 degrees of any mode are the same notational objects
                           (diatonic()). spelled() turns a degree into an SPN spelling
                           (root letter stepped degree-1 letters up) and panics when
                           the spelling would need more than a double sharp/flat
  mode.rs                  7 diatonic modes. A mode is a *rotation* of the major
                           scale — root_offset_semitones() indexes into
                           [0,2,4,5,7,9,11], pitch_classes() rotates the parent scale
  key.rs                   {tonic: SpelledPitch, Major|Minor}. mode_root() lifts the
                           tonic to any mode via the parent-major trick and returns a
                           *spelled* pitch; mode_pitch_classes() is the scale as
                           pitch classes; mode_of_quality() maps Major/Minor to
                           Ionian/Aeolian
  scales.rs                degree-subset presets: pentatonic_major/minor (1 2 3 5 6 /
                           1 3 4 5 7), arpeggio_major/minor (triads; the minor preset
                           is the same 1 3 5 — scale-relative degrees already spell the
                           minor triad). These lists drive highlighting and shapes.
  degree_reference.rs     {scale_degree, octave} — THE abstract note, and the chunk
                           layer's domain type. Octave is TONIC-anchored: octave 0
                           spans from the tonic up to the octave above, so degree 1
                           octave 0 IS the tonic. interval() is THE resolution primitive
                           (semitones above the tonic); note() builds an absolute SPN
                           note on top of it — the seam where relative becomes absolute.
  chunk.rs                 the movable pattern layer, most conceptually central:
                           Slot::Rest | Slot(DegreeReference) (slot presence IS the
                           rhythm; all slots equal duration — deliberate
                           simplification); Chunk = Vec<Slot>; reframe() rotates
                           degrees mod 7 with octave carry — this is what makes chunks
                           transposable; apply_progression() repeats a chunk per chord
                           root (e.g. [1, 4, 5])

crates/fretboard/src/      where you PLAY it — relative + absolute → strings and frets
  fretboard_position.rs    {string_index: usize, fret: i32} — a concrete position
  tuning.rs                open strings as absolute SPN notes (Tuning = Vec<Note>;
                           standard = E2 A2 D3 G3 B3 E4), low to high. Absolute pitch is
                           the point: pitch classes cannot distinguish E2 from E3
                           (Nashville tuning, bass, octave-displaced tunings).
                           retune_string() shifts one string and spells minimally;
                           presets (standard, drop_d, standard_seven_string, bass_four)
                           derive from standard
  instrument.rs            tuning + fret count *per string*. Everything that asks
                           "can this be played here" (playable, range) goes through an
                           instrument; semitone_at() resolves a position to sounding
                           pitch. This is where fret availability lives — not in view
                           config.
  fretboard.rs             the slim bridge: positions_of_semitone (the one position
                           enumerator, shared with the solver), degree_positions()
                           (degree → positions, instrument-clamped), degree_at()
                           (position → Option<DegreeReference>, None off-key),
                           degrees_in_window() (every in-key playable position WITH
                           its degree). root_note() is the octave register all
                           resolution here is anchored to; degree_at round-trips
                           through DegreeReference::interval.
  shape.rs                 notes-per-string shapes over ANY degree list — the diatonic
                           degrees give the seven 3NPS positions, pentatonic_major's
                           the five 2NPS, arpeggio's the three 1NPS inversions.
                           Shapes drop to their lowest playable octave; unplayable
                           shapes are omitted.
  reach.rs                 finger-stretch geometry: fret span, string span, and the
                           sounding interval span between two positions
                           (semitone_span is tuning-aware — cross-string stretch
                           depends on the tuning's intervals). Only between() is wired
                           so far (Chunk_solver's distance); ranking by semitone_span
                           awaits the pick-and-practice flow.
  chunk_solver.rs          positions() — every fingering of a chunk, ranked by reach
                           distance from a start anchor. CRITICAL invariant: resolve
                           each note's absolute semitone (via Fretboard::degree_note)
                           FIRST, then choose only *which string* — the comment at the
                           call site says why. The first note's octave window is
                           derived from Instrument::range — no magic sweep constants —
                           and candidate enumeration reuses
                           Fretboard::positions_of_semitone. Lives here, not beside
                           Chunk: it maps chunks onto the instrument.
  bin/dump_fixtures.rs     prints canonical solver/shape/window fixtures across keys and
                           tunings. The OCaml counterpart it was diffed against is gone
                           with the OCaml tree; keep this as the regression corpus for
                           future refactors and re-baseline it deliberately.
```

## Data flow

Chunk path (library code, no UI yet): `chunk::apply_progression` (harmonic
resolution per chord) → `chunk_solver::positions` (every fingering, sorted by
distance) → whoever calls it (no GUI yet; todo: pick-and-practice flow).

The previous GUI's per-frame flow (recorded in `docs/rust-migration-plan.md`):
state indices → key/mode → `fretboard::degrees_in_window` / `shape::positions`
(concrete positions, clamped to the instrument) → tab + fretboard rendering.

## Conventions that will bite you if ignored

House style:

- **No comments** unless the code can't say it (a *why* a reader can't get
  otherwise). Existing comments of that kind: the hand-written Ord impls, the
  tonic-anchored octave, the solver's resolve-then-choose-string invariant —
  match that bar.
- Top-down: the orchestrating function reads like a summary; build the pieces
  bottom-up.
- Functional first; modules don't know about each other's callers.
- Tests are inline (`#[cfg(test)]` per module) and mirror the crate layers;
  every module gets a suite. Port/extend cases with the same expected-value
  style the existing tests use.
- Test the computations, not the shell: no window, no GUI init in tests.
  Don't add a test a type error or five-second manual check wouldn't catch.
- **Layer discipline is compiler-enforced** by the workspace dependency graph.
  Treat any need to reach upward as a design bug — add the function to the
  layer that owns the data instead.
- Run `cargo fmt` before calling something done. Warnings aren't noise; if it
  doesn't build clean, it isn't done.
- `Option` for genuinely-absent results; `panic!` only for invariant violations,
  with the invariant documented on the function that panics.
- Small value types are `Copy` (PitchClass, Letter, Alteration, SpelledPitch,
  Note, FretboardPosition, Reach, ScaleDegree, DegreeReference, Slot).
- Tuning/Chunk are type aliases over `Vec` with module-level functions;
  structs (Instrument, Key) carry methods.
- No persistence anywhere yet — a deliberate deferral, not an oversight.
