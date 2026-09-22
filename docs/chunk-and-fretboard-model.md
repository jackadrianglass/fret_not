# Chunk and fretboard model

How this project gets from "a movable musical idea" to "concrete frets,"
and back. This is a reference for the model, not a tutorial on the code —
read [`coding-guidelines.md`](coding-guidelines.md) for how it's written.
Update this file when the model changes; it should stay in sync with
`lib/`, not describe an earlier version of it.

## 1. The idea: a chunk is abstract on purpose

The [readme](../readme.md) calls this "the architectural core of the whole
project." A **chunk** is a sequence of notes (and rests) defined by scale
degree *relative to a key and mode* — not by fixed frets, and with no
separate rhythm representation of its own (see §7). Because it's abstract,
the same chunk can be projected onto the fretboard in any key, at any
position on the neck, or extracted back out of a tab someone's learning.

Two things have to both be true for that to work:

- The abstract side (key, mode, scale degree) has to know nothing about
  strings or frets.
- The concrete side (tuning, fretboard position) has to know nothing about
  scale degrees.

Only a small number of things are allowed to touch both: `Fretboard`, the
original projection bridge (§5), and `Chunk_solver` (§7), which builds on
it to solve a whole sequence at once. Everything else stays on one side or
the other. That split is the whole model.

`Chunk` itself is built now — see §7 — on top of the foundation described
next: the abstract unit a chunk is a sequence *of*, and the bridge that
projects it.

## 2. The abstract side: Key, Mode, Scale Degree

- **`Key.t`** (`lib/key.ml`) — a tonic pitch class plus a `Major`/`Minor`
  quality. Nothing else. A key doesn't pick a register (no octave) — it's
  meant to be movable to any key by construction.
- **`Mode.t`** (`lib/mode.ml`) — one of the 7 diatonic modes (Ionian,
  Dorian, Phrygian, Lydian, Mixolydian, Aeolian, Locrian). All 7 are
  rotations of one interval formula, not 7 separate ones: each mode's
  tonic sits at a fixed semitone offset from its parent major scale's
  root, and its degrees are that parent scale rotated to start there.
- **`Key.modes`** — the "give me everything a major/minor key implies"
  entry point: all 7 `(Mode.t, Scale_degree.t list)` pairs at once. A
  major key and its relative minor produce identical output — same
  notes, different framing. For C major:

  | Mode | Degrees |
  |---|---|
  | Ionian | C D E F G A B |
  | Dorian | D E F G A B C |
  | Phrygian | E F G A B C D |
  | Lydian | F G A B C D E |
  | Mixolydian | G A B C D E F |
  | Aeolian | A B C D E F G |
  | Locrian | B C D E F G A |

- **`Scale_degree.t`** (`lib/scale_degree.ml`) — `{ degree; pitch_class }`.
  `degree` is 1-based position within the mode. Still fully abstract — no
  octave, no fret, just "which note of the mode, and what pitch class
  does it land on for this key."

## 3. The atomic unit: Degree Reference

**`Degree_reference.t`** (`lib/degree_reference.ml`) — `{ degree; octave;
alteration }`. This is what a chunk will eventually be a sequence *of*.

- `degree` — which of the mode's notes (1-7 today; pentatonic subsets are
  deliberately deferred, see `tasks/00-foundations/05-pentatonic-scale-subsets.md`).
- `alteration` — a semitone offset (plain `int`, not a constrained enum —
  the only sane representation on a fretted, equal-tempered instrument).
  Lets a Degree Reference describe a chromatic note that isn't literally
  in the mode, e.g. a flatted 5th as a blue-note passing tone.
- `octave` — **not an absolute register.** This is the one piece of the
  model that took real back-and-forth to get right (see
  `tasks/00-foundations/03-fretboard-position-projection.md`'s Notes for
  the full reasoning). The first design tied it to real scientific pitch
  (true E2/E4), which quietly breaks movability: a chunk's own data would
  always resolve to the same physical notes no matter where you tried to
  project it. Instead, octave is meaningful only *relative to wherever
  you're projecting from* — see §5.

## 4. The concrete side: Tuning, Fretboard Position

- **`Tuning.t`** (`lib/tuning.ml`) — an ordered list of open-string pitch
  classes, low to high, arbitrary length (a 6-string and a 7-string
  tuning are both just a different-length list, no model changes).
  Deliberately stores pitch classes only, no octave — see §5 for why that
  turned out to be enough.
- **`Fretboard_position.t`** (`lib/fretboard_position.ml`) —
  `{ string_index; fret }`. `string_index 0` is the lowest-pitched
  string.
- **`Tuning.relative_semitone`** — true semitone height above
  `string_index 0`'s open pitch (not modulo 12), derived on demand from
  the existing pitch-class list rather than stored. Every real guitar
  tuning has each string's open pitch a small ascending interval above
  the previous one, so this is just a running sum of those gaps.

## 5. The bridge: anchoring a projection

**`Fretboard.to_positions`** and **`Fretboard.of_position`**
(`lib/fretboard.ml`) are the only things that touch both sides. Both take
an `anchor_position : Fretboard_position.t`, and it means the same thing
in both directions: **"octave 0" is the nearest occurrence of the target
pitch class to that position** (ties break upward). Octave `N` shifts by
`12 * N` semitones from there.

That's what makes a chunk movable: the same, unedited `Degree_reference.t`
resolves to a different physical spot just by changing which
`anchor_position` you project it from. Moving a chunk up the neck is a
different function call, not a data mutation.

Mechanically: each string reaches a given target pitch at *exactly one*
fret, or not at all (pitch rises monotonically with fret, no wraparound
within a string) — so `to_positions` returns one candidate per reachable
string, sorted by fret-distance from the anchor, with no fret bound (a
real neck's limit belongs to the render surface, not this module).
`of_position` runs the same anchor math in reverse, and currently raises
if the position isn't on one of the mode's 7 diatonic pitch classes —
chromatic reverse-spelling is deferred (§7).

## 6. Worked examples

All in the key of **C major, Ionian**, tuning **standard** (E A D G B E).

### Example A — one Degree Reference, multiple fingerings

`{ degree = 1; octave = 1; alteration = 0 }` (the tonic, one octave above
the anchor) projected from `anchor_position = { string_index = 0; fret =
0 }` (open low E):

```
resolves to: [ (string 1 "A", fret 3); (string 0 "low E", fret 8) ]
```

Both are the same physical note (C3) — the A string open plus 3 frets,
or the low E string plus 8. `to_positions` returns both, nearest to the
anchor first.

### Example B — the same note, described relative to a different anchor

Now anchor at `{ string_index = 0; fret = 8 }` (that same C3 on the low E
string) instead, and ask for `{ degree = 1; octave = 0; alteration = 0 }`
— octave 0 this time, since we're already anchored on the tonic itself:

```
resolves to: [ (string 0 "low E", fret 8); (string 1 "A", fret 3) ]
```

Identical set of physical positions as Example A. The only thing that
changed is which anchor and which octave number describe them — proof
that octave really is anchor-relative, not an absolute fact about the
note.

### Example C — alteration, for a chromatic passing tone

`{ degree = 5; octave = 0; alteration = -1 }` (a flatted 5th — F#, not in
C major's own 7 notes) from the same open-low-E anchor:

```
resolves to: [ (string 0 "low E", fret 2) ]
```

`alteration` is what lets a Degree Reference reach outside the diatonic
set without needing a different mode.

## 7. The Chunk layer: Slot, Chunk, Chord Progression

Settled in `worklog/08-chunk-model-design/` (design conversation) and
landed in `worklog/09-current/` (the code). A **Chunk** is a movable,
degree-relative sequence of notes — the thing §1 promised but didn't build
yet is now built, one layer up from a single `Degree_reference.t`.

- **`Slot.t`** (`lib/chunk/slot.ml`) — `Rest | Note of Degree_reference.t`.
  Deliberately no separate rhythm type: a slot's presence/absence *is* the
  rhythm, and every slot is equal duration. "Applying a Chunk over a
  subdivision" (e.g. a 4-slot chunk over a 3-grouping, a 4-over-3
  polyrhythm) means picking how many real time-grid units the whole slot
  list spans — a number at render/practice time, not a new object here.
- **`Chunk.t`** (`lib/chunk/chunk.ml`) — `Slot.t list`. No key/mode stored
  on the Chunk itself, same as `Degree_reference.t` — a Chunk's degree
  numbers only mean something once paired with a Key+Mode at projection
  time, via the existing `Fretboard` bridge.
- **`Chunk.reframe`** — reinterprets every `Note` slot's own degree number
  against a new root degree (1-7), the same rotate-by-N-degrees move
  `Mode.degrees` makes when picking which pitch class counts as a mode's
  own root, generalized to an arbitrary scale-degree anchor. A degree that
  wraps past 7 back around to 1 carries an octave into the slot's own
  `Degree_reference.octave`, since that field is a real ±12-semitone
  offset once projected through `Fretboard`, not just a label — a bare
  `mod 7` relabel without the carry would silently drop a wrapped note an
  octave low.
- **`Chord_progression.t`** (`lib/chunk/chord_progression.ml`) —
  `int list` of scale-degree roots (e.g. `[ 1; 4; 5 ]` for I-IV-V). Its
  own movable object, not a Song attribute — a progression maps cleanly
  onto a Song but not the reverse, and one Song often has several. Chord
  quality is inferred diatonically from whatever Key+Mode it's paired
  with; no explicit per-step quality override (borrowed chords, secondary
  dominants) yet.
- **`Chord_progression.apply_to_chunk`** — one reframed Chunk per
  progression step, via `Chunk.reframe`. E.g. a bare-tonic chunk (every
  `Note` slot at degree 1) applied over `[ 1; 4; 5 ]` produces three
  chunks, at degrees 1, 4, and 5 respectively.

- **`Chunk_solver.positions`** (`lib/chunk/chunk_solver.ml`) — the first
  real Chunk-to-fretboard solver, and the second thing (after `Fretboard`
  itself) that touches both the abstract and concrete sides directly: it
  takes a starting anchor and a sequence of already-harmonically-resolved
  `Degree_reference.t` (one chord's own reframed triad, say), and returns
  *every* way to realize them as concrete positions that stay within a
  hard per-move fret cap — `Chunk_solver.distance`'s `|fret delta| +
  |string delta|` between consecutive notes, sorted nearest-overall
  first. It resolves each note's own real, ever-increasing semitone height
  (pitch class from `Fretboard.target_pitch_class`, plus 12 semitones per
  octave the `Degree_reference` itself carries) *before* choosing any
  fretboard position — reframing only ever shifts every note in a chunk by
  the same amount, so two notes a Chunk was authored an interval apart
  keep that exact interval once reframed onto a new chord, wrapped octave
  or not. Only *which string* reaches that already-determined pitch is
  chosen positionally, within the fret cap of wherever the previous note
  landed. An earlier version of this ignored `Degree_reference.octave`
  entirely to dodge a cross-chord compounding bug (see below) — that
  broke every non-tonic chord in a progression (a reframed note that had
  wrapped an octave would resolve a full octave off, breaking the
  intended ascent) and was corrected; ignoring octave was the wrong fix; the
  real fix was resolving every note's own true octave from its harmonic
  content instead of chaining physical anchors and hoping proximity alone
  preserved it.

Not part of this layer yet: modal-position-constrained solving,
variant/lineage tracking, tagging, and persistence — all still `todo.md`
bullets.

## 8. Open threads (not decided yet)

- **Modal-position-constrained solving.** `Chunk_solver` finds the
  hand-comfortable realization of a chunk's own exact harmonic content: it
  doesn't (and structurally can't) offer "stay within this known
  scale-position box instead" as a different, equally valid choice — see
  `worklog/09-chunk-library-core` for a concrete case where a chunk's
  shared-anchor projection happened to also fit cleanly inside an existing
  `Fretboard` notes-per-string position, before `Chunk_solver` existed. A
  second solver strategy, constrained to one of those positions, is still
  undesigned. See `todo.md`'s Chunk-to-Fretboard Solvers section.
- **Chromatic reverse spelling.** `of_position` currently raises on a
  non-diatonic pitch rather than guessing a spelling (sharp of the degree
  below vs. flat of the degree above are enharmonically equivalent but
  not interchangeable as data). Deferred to whichever task first needs to
  extract a chromatic passing tone from a real tab — probably the
  Permutation Engine or Tab-to-Chunk Extraction sections of `todo.md`.
- **Pentatonic and other subsets.** Not modes of their own — subsets of
  Ionian/Aeolian's own degrees. See
  `tasks/00-foundations/05-pentatonic-scale-subsets.md`.

## 9. Code map

`lib/` is grouped into five folders matching §2/§4/§5/§7 above —
`theory/` (abstract, no notion of a fretboard at all), `fretboard/`
(concrete + the bridge), `chunk/` (movable degree-relative sequences,
built on top of `Degree_reference` but with no fretboard knowledge of its
own), `layout/` (pure rendering-support geometry, still no raylib),
`view/` (pure UI state/config for the GUI shell, still no raylib). All
are one dune library (`include_subdirs unqualified`), so module names
stay unqualified — `Key`, `Mode`, `Tuning`, etc. — only the file location
changes. `test/` mirrors the same folders, one test file per source
module. The raylib/raygui-touching drawing code itself lives outside this
library, in `bin/fretboard_view.ml` — see `contributing/coding-guidelines.md`
for why that boundary matters.

| Concept | Module |
|---|---|
| Pitch class (0-11) | `lib/theory/pitch_class.ml` |
| Key (tonic + major/minor) | `lib/theory/key.ml` |
| The 7 diatonic modes | `lib/theory/mode.ml` |
| One note of a mode, in context | `lib/theory/scale_degree.ml` |
| Open-string tuning | `lib/fretboard/tuning.ml` |
| A concrete string+fret | `lib/fretboard/fretboard_position.ml` |
| An abstract degree+octave+alteration | `lib/fretboard/degree_reference.ml` |
| The abstract ↔ concrete bridge | `lib/fretboard/fretboard.ml` |
| A rest-or-note step in a Chunk | `lib/chunk/slot.ml` |
| A movable, degree-relative sequence of Slots | `lib/chunk/chunk.ml` |
| A movable sequence of scale-degree chord roots | `lib/chunk/chord_progression.ml` |
| The Chunk-to-fretboard positional solver | `lib/chunk/chunk_solver.ml` |
| Pixel geometry for rendering (not domain) | `lib/layout/fretboard_layout.ml` |
| Control-row geometry (x-positions, dropdown width) | `lib/layout/row_layout.ml` |
| GUI presentation config (canvas size, radii, fonts, ...) | `lib/view/fretboard_view_config.ml` |
| GUI selection/state computations (scale, position, labels) | `lib/view/fretboard_view_state.ml` |
