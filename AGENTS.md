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
- Relative music layer (`crates/relative/src`) defines what a scale degree, key, and word are
    — a word is a movable sequence of degree-relative notes, and resolves degrees
      against a key into absolute notes.
- Fretboard layer (`crates/fretboard/src`) maps those onto concrete string/fret positions
  for a given instrument, including finding every playable fingering of a word.
- The GUI layer (`crates/gui/src`, iced 0.14) implements the app's feature
  set; `docs/gui-spikes.md` records the checklist it was
  built against and `docs/gui-feature-capture.md` the feature capture.

Dependencies only point downward, enforced by the cargo workspace layout: `relative`
depends on `absolute`, `fretboard` depends on both, and nothing below imports upward.
The GUI crate sits beside `fretboard` and may import everything.

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
cargo fmt                 # rustfmt; don't call. happens automatically when changes are committed
cargo build --workspace   # build
cargo clippy --workspace  # stricter lint pass
```

Building, checking, formatting, testing, and linting are allowlisted — run
them freely.
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
                           goes through from_int and the Add/Sub impls (rem_euclid,
                           so negatives wrap)
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
                           (ScaleDegree::DIATONIC). spelled() turns a degree into an
                           SPN spelling (root letter stepped degree-1 letters up) and
                           panics when the spelling would need more than a double
                           sharp/flat
  mode.rs                  7 diatonic modes. A mode is a *rotation* of the major
                           scale — root_offset_semitones() indexes into
                           [0,2,4,5,7,9,11], pitch_classes() rotates the parent scale
  key.rs                   {tonic: SpelledPitch, Major|Minor}. mode_root() lifts the
                           tonic to any mode via the parent-major trick and returns a
                           *spelled* pitch; mode_pitch_classes() is the scale as
                           pitch classes; From<Quality> for Mode maps Major/Minor to
                           Ionian/Aeolian
  scales.rs                degree-subset presets as const arrays: PENTATONIC_MAJOR/
                           MINOR (1 2 3 5 6 / 1 3 4 5 7), ARPEGGIO_MAJOR/MINOR (triads;
                           the minor preset is the same 1 3 5 — scale-relative degrees
                           already spell the minor triad). These lists drive
                           highlighting and shapes.
  degree_reference.rs     {scale_degree, octave} — THE abstract note, and the word
                           layer's domain type. Octave is TONIC-anchored: octave 0
                           spans from the tonic up to the octave above, so degree 1
                           octave 0 IS the tonic. interval() is THE resolution primitive
                           (semitones above the tonic); note() builds an absolute SPN
                           note on top of it — the seam where relative becomes absolute.
  word.rs                 the movable pattern layer, most conceptually central:
                           Slot::Rest | Slot(DegreeReference) (slot presence IS the
                           rhythm; all slots equal duration — deliberate
                           simplification); Word is a newtype over Vec<Slot>
                           (Deref to the slice; push/remove/set as module functions);
                           reframe() rotates degrees mod 7 with octave carry — this is
                           what makes words transposable; apply_progression() repeats
                           a word per chord root (e.g. [1, 4, 5])

crates/fretboard/src/      where you PLAY it — relative + absolute → strings and frets
  fretboard_position.rs    {string_index: usize, fret: i32} — a concrete position
  tuning.rs                open strings as absolute SPN notes (Tuning is a newtype over
                           Vec<Note>, Deref to the slice; standard = E2 A2 D3 G3 B3
                           E4), low to high. Absolute pitch is the point: pitch
                           classes cannot distinguish E2 from E3 (Nashville tuning,
                           bass, octave-displaced tunings). retune_string() shifts one
                           string and spells minimally; presets (standard, drop_d,
                           standard_seven_string, bass_four) derive from standard
  instrument.rs            tuning + fret count *per string*. Everything that asks
                           "can this be played here" (playable, range) goes through an
                           instrument; semitone_at() resolves a position to sounding
                           pitch. This is where fret availability lives — not in view
                           config.
  fretboard.rs             the slim bridge: positions_of_semitone (the one position
                           enumerator, shared with the solver), degree_positions()
                           (degree → positions, instrument-clamped), degree_at()
                           (position → Option<DegreeReference>, None off-key),
                           spelled_degree_at() (the total form: every position
                           resolves, off-key ones as altered degrees — Eb in C
                           major is b3; in-key results match degree_at),
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
                           so far (Word_solver's distance); ranking by semitone_span
                           awaits the pick-and-practice flow.
  word_solver.rs          positions() — every fingering of a word, ranked by reach
                           distance from a start anchor; positions_from() — the
                           same with the first note pinned to an exact position
                           (the edit screen's rotation catalog);
                           starting_position() — the nearest playable position
                           sounding a note (a never-placed root's default).
                           CRITICAL invariant: resolve each note's absolute semitone (via
                           Fretboard::degree_note) FIRST, then choose only
                           *which string* — the comment at the call site says
                           why. The first note's octave window is derived from
                           Instrument::range — no magic sweep constants — and
                           candidate enumeration reuses
                           Fretboard::positions_of_semitone. Lives here, not
                           beside Word: it maps words onto the instrument.
  fingering.rs             Fingering — one concrete way to play a sequence of
                           notes, a position per note in note order (newtype
                           over Vec, Deref to the slice). slot_positions()
                           projects a fingering back onto slot-aligned
                           positions (None for rests) via the slot-to-note
                           map; distance()/total_distance() are the reach
                           metrics fingerings are ranked by
  bin/dump_fixtures.rs     prints canonical solver/shape/window fixtures across keys and
                           tunings; the regression corpus for future refactors —
                           re-baseline it deliberately.

crates/gui/src/            where you SEE it — the iced 0.14 GUI (layout prototype)
  main.rs                  the shell: App = Session (instrument/tonic/quality) +
                           Settings (theme/label mode/font size/solver fret cap) +
                           Document (words as WordEntry — a word plus its
                           variant_of lineage, flat under one root — and sequence
                           steps, owned here and shared by every screen) + Playback;
                           persistent chrome
                           (top bar with nav, key/tempo config and playback
                           groups, collapsible word-library rail — hidden on the
                           settings page); screen views are routed in via .map
                           and report mutations back as a bool that triggers
                           screens.refresh
  board.rs                 the shared fretboard component: BoardState (in-key dots,
                           recomputed on session/label-mode change in update) +
                           Highlight (played positions + one optional ringed
                           position); emits board::Event — each screen maps it
                           into its own Message
  colors.rs                palette-driven custom colors: every chrome/board shade is
                           derived from the iced Theme palette at render time, inside
                           widget style closures and the canvas draw call — theme
                           switching only reassigns state.settings.theme, no
                           refresh anywhere
  screens/                 one module per feature view, each with a private Message;
                           fingerings are cached in update and go stale on any
                           document/session/settings change, which the shell fixes
                           by calling Screens::refresh
    edit.rs                the edited word's family stacked as slot bands (root
                           first, then each variant; label at left, the edited band
                           accent-bordered and -labeled): every band is fully
                           editable inline, and the board, stepper and rest toggle
                           follow the edited word; "+ add variant" copies the
                           edited word (slots + authored positions + octave base)
                           parented to the root; board clicks write degrees into
                           the edited word's selected slot, off-key clicks clear
                           it to a rest
    tab.rs                  tab lines primary; chord/fingering steppers above, compact
                           board below; clicking a board dot jumps to a fingering
                           that plays it
    sequence.rs            step-card timeline (root roman + word letter + degree
                           glyphs, horizontally scrollable); the board follows the
                           selected step
    settings.rs            the settings page: rule-divided sections (Appearance:
                           theme/labels/font size, Fretboard: tuning preset +
                           solver fret cap) over the docked board as a live
                           preview; stateless but for the number inputs' text
  model.rs                 view-state glue only: TONICS const + typed pick_list
                           payloads (InstrumentPreset, LabelMode), BASE_FONT_SIZE,
                           board assembly over degrees_in_window +
                           spelled_degree_at (in-key dots and accidental dots),
                           solve wiring, seed words/steps, word names from the
                           lineage (roots lettered, variants numbered under their
                           root), romans — music knowledge stays in the core crates
  fretboard_canvas.rs      canvas Program: grid, dots, halos, hit-testing; paints
                           from theme-derived CanvasColors, no state, no animation —
                           fret N's dot centers between the N-1 and N wires (where
                           the finger presses), fret 0 sits left of the nut;
                           strings are drawn tab-convention (high E top) with SPN
                           open-note labels, degree labels carry a small key-referenced
                           octave-count subscript; every text size scales off the
                           font-size setting
```

## Data flow

Word path: `word::apply_progression` (harmonic resolution per chord) →
`word_solver::positions` (every fingering, sorted by distance) → the GUI's
pick-and-practice flow (`model::solve`, prev/next over the ranked fingerings).

GUI flow: tonic/quality → key/mode → `model::board` over
`fretboard::degrees_in_window` / `degree_at` (concrete positions, clamped to
the instrument) — computed in update on session/settings change and cached in
BoardState; each screen caches its solved fingerings and the shell calls
Screens::refresh after any document/session/settings mutation → tab + fretboard
rendering. The fretboard is a shared component docked at the bottom of every
screen: the full in-key field drawn as a quiet labeled backdrop, the played
positions emphasized on top of it (a live preview of the label-mode and
font-size settings on the settings page).

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
- `cargo fmt` reflows files: after running it, re-read any file before
  editing it again — a stale read makes the next `edit` old_string miss.
- `Option` for genuinely-absent results; `panic!` only for invariant violations,
  with the invariant documented on the function that panics.
- Small value types are `Copy` (PitchClass, Letter, Alteration, SpelledPitch,
  Note, FretboardPosition, Reach, ScaleDegree, DegreeReference, Slot).
- Tuning/Word are newtypes over `Vec` (Deref to the slice) with module-level
  functions; structs (Instrument, Key) carry methods.
- No persistence anywhere yet — a deliberate deferral, not an oversight.
