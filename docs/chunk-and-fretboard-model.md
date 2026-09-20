# Chunk and fretboard model

How this project gets from "a movable musical idea" to "concrete frets,"
and back. This is a reference for the model, not a tutorial on the code —
read [`coding-guidelines.md`](coding-guidelines.md) for how it's written.
Update this file when the model changes; it should stay in sync with
`lib/`, not describe an earlier version of it.

## 1. The idea: a chunk is abstract on purpose

The [readme](../readme.md) calls this "the architectural core of the whole
project." A **chunk** is a sequence of notes defined by scale degree and
rhythm *relative to a key, mode, and time signature* — not by fixed frets.
Because it's abstract, the same chunk can be projected onto the fretboard
in any key, at any position on the neck, or extracted back out of a tab
someone's learning.

Two things have to both be true for that to work:

- The abstract side (key, mode, scale degree) has to know nothing about
  strings or frets.
- The concrete side (tuning, fretboard position) has to know nothing about
  scale degrees.

Exactly one thing is allowed to touch both: the projection bridge. That
split is the whole model.

`Chunk` itself isn't built yet (`tasks/04-chunk-library/`) — what exists
today is the foundation it'll sit on: the abstract unit a chunk is a
sequence *of*, and the bridge that projects it.

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

## 7. Open threads (not decided yet)

- **How a multi-note chunk anchors itself.** `to_positions` takes one
  anchor per call. A real chunk is a *sequence* of Degree References — do
  all of them anchor to the same single position (as in the examples
  above), or does each note anchor to wherever the *previous* note
  landed, chaining down the phrase? Both are reasonable; this is
  `04-chunk-library`'s call to make with real chunk examples in hand.
- **Chromatic reverse spelling.** `of_position` currently raises on a
  non-diatonic pitch rather than guessing a spelling (sharp of the degree
  below vs. flat of the degree above are enharmonically equivalent but
  not interchangeable as data). Deferred to whichever task first needs to
  extract a chromatic passing tone from a real tab — probably
  `05-permutation-engine` or `06-tab-to-chunk-extraction`.
- **Pentatonic and other subsets.** Not modes of their own — subsets of
  Ionian/Aeolian's own degrees. See
  `tasks/00-foundations/05-pentatonic-scale-subsets.md`.

## 8. Code map

`lib/` is grouped into three folders matching §2/§4/§5 above — `theory/`
(abstract, no notion of a fretboard at all), `fretboard/` (concrete +
the bridge), `layout/` (pure rendering-support geometry, still no
raylib). All are one dune library (`include_subdirs unqualified`), so
module names stay unqualified — `Key`, `Mode`, `Tuning`, etc. — only the
file location changes. `test/` mirrors the same three folders, one test
file per source module.

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
| Pixel geometry for rendering (not domain) | `lib/layout/fretboard_layout.ml` |
