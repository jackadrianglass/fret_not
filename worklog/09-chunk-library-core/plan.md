# Plan

Implementing the base `Chunk` model that `worklog/08-chunk-model-design/`
settled on ("Model sketch" section) — this entry is the actual code, not
another design pass. Scope is deliberately narrow: the library-side data
model and reprojection logic only. No UI, no persistence, no solver.

# Context

`worklog/08-chunk-model-design/plan.md`'s Model sketch is the spec:

- `Slot.t = Rest | Note of Degree_reference.t`, `Chunk.t = Slot.t list`. No
  separate rhythm type — a slot's presence/absence is the rhythm, every slot
  equal duration.
- `Chord_progression.t = int list` of scale-degree roots (e.g. `[ 1; 4; 5 ]`
  for I-IV-V). Quality inferred diatonically — no per-step override yet.
- Reprojecting a Chunk over one progression step reuses the same
  rotate-by-N-degrees move `Mode.degrees` makes when picking a mode's own
  root: the step's scale degree becomes a fresh local anchor each of the
  Chunk's own degree numbers gets reinterpreted against, wrapping mod 7.

Explicitly out of scope here (per `todo.md`'s Chunk-to-Fretboard Solvers /
Permutation Engine / persistence sections): fretboard projection strategies,
variant/lineage tracking, tagging, persistence, any UI.

# Todo

- [x] `Slot.t` (`lib/chunk/slot.ml`/`.mli`)
- [x] `Chunk.t` + `Chunk.reframe` (`lib/chunk/chunk.ml`/`.mli`)
- [x] `Chord_progression.t` + `Chord_progression.apply_to_chunk`
      (`lib/chunk/chord_progression.ml`/`.mli`)
- [x] Tests for all three, wired into `test/test_fret_not.ml`
- [x] `dune build @fmt --auto-promote` + `dune test`
- [x] Sync `docs/chunk-and-fretboard-model.md` and `todo.md`
- [x] `Chunk_solver.positions` (`lib/chunk/chunk_solver.ml`/`.mli`) - a real,
      tested positional solver (see Questions' "Positional solver design"
      below), replacing the demo's ad hoc chaining
- [x] Tests for `Chunk_solver`, wired into `test/test_fret_not.ml`
- [x] Re-wired `bin/main.ml`'s demo to solve the full I-vi-IV-V progression
      per chord via `Chunk_solver`, no more cross-chord chaining

# Questions

## Octave carry on wraparound

The design doc's phrasing ("wrapping mod 7") doesn't explicitly say what
happens to a Chunk slot's own `octave` field when reframing pushes its degree
past 7 back around to 1. Concretely: a chunk's degree-5 slot (a 5th above its
own root) reframed onto progression step 5 (root degree 5) lands on
`(5-1)+(5-1) = 8`, i.e. one full wrap past the top of the scale — musically
that's genuinely an octave higher than a bare `mod 7` relabel would place it,
since `Degree_reference.octave` is a real ±12-semitone offset once projected
through `Fretboard`, not just a label.

### Answer

Carrying the octave (`reframed.octave = original.octave + (raw_degree / 7)`)
is the only reading that keeps `Chunk.reframe` correct once a reframed slot
actually gets projected via `Fretboard.to_positions` — a same-anchor
`Fretboard` projection is fully accurate about a degree reference. No
answer needed to proceed; noting this here since the design doc alone was
silent on it, before deviating from a "no comments" trap in the code.

## Positional solver design

Running the demo surfaced a real bug: projecting every slot of a plain
root-3rd-5th Chunk independently from one fixed anchor let the 5th resolve
*below* the root and 3rd (nearest-occurrence-to-one-anchor doesn't track
melodic direction). A first fix (chaining each note off wherever the
previous one landed) worked for a single triad, but chaining continuously
across a whole `Chord_progression` compounded badly with `Chunk.reframe`'s
own octave carry, spiraling up the neck within a few chords.

Talking through the actual mental model clarified the fix: a Chunk
projects in two genuinely separate stages -

1. **Harmonic resolution** (already built, `Chunk.reframe` +
   `Fretboard`'s own pitch-class resolution): scale-degree numbers in,
   the correct scale-degree numbers out *per chord*, automatically
   respecting that chord's own quality (a Chunk's "1, 3, 5" reframed onto
   a minor-quality step correctly comes out as a minor triad, not a major
   shape transposed - confirmed with the I-vi-IV-V demo, vi resolved to
   A-C-E). This stage has no fretboard knowledge and needs no changes.
2. **Positional solving** (not built until now): given one chord's own
   resolved degrees, find actual, comfortable fretboard positions -
   reachable, and close enough together to actually play. This is a
   separate job from stage 1, and shouldn't reuse a Chunk's own
   `Degree_reference.octave` field at all, since that field's original
   "nearest occurrence to a fixed anchor" meaning doesn't compose with a
   bounded, chained search the way stage 1 needs it to.

### Answer

Distance metric: raw `|fret delta| + |string delta|` between two
positions - no same-string bonus (confirmed after checking a same-string
candidate the user offered as an example turned out not to be the
geometrically-closest one anyway; the user confirmed that specific
example wasn't a deliberate preference, just "started in an Ionian
shape"). A hard cap of 7 frets on any single move *within* one chunk
instance (a real hand-span limit, not just a ranking preference) - jumping
between separate instances of the chunk (e.g. between chords) is a
"position change," not a stretch, and isn't bounded by this at all, since
each chord's instance is solved fresh from the same `start_anchor` rather
than chained from the previous chord's ending note.

`Degree_reference.octave` is deliberately ignored by `Chunk_solver`: since
every note's candidates are already re-searched fresh (within
`max_fret_distance` of wherever the previous note landed, across whatever
octave that requires), the octave value `Chunk.reframe` computed for stage
1 would just double up with a register the bounded search already finds
on its own - which is exactly what caused the compounding bug. The
tradeoff: a Chunk's own deliberately large melodic leap (e.g. a note
written a full octave above the one before it, as in the very first
demo's "top" note) isn't preserved by this solver - flagged as still open
in `todo.md` rather than solved here, since it wasn't needed for the
`[1; 3; 5]` case this pass focused on.

The user also wants to eventually browse *multiple* solved shapes rather
than commit to just one ("we can resolve to multiple shapes and pick one
from the batch to practice") - `Chunk_solver.positions` already returns
every valid shape sorted by total distance, not just the best one; only
`bin/main.ml`'s demo currently throws the rest away by taking the head of
the list. Picking one from the batch to render/practice is real
`todo.md` Chunk-to-Fretboard Solvers scope, not done here.

## Correction: ignoring octave was wrong

The "ignore `Degree_reference.octave` entirely" answer above was wrong,
caught by eyeballing a real I-vi-IV-V run: the i chord (root-3rd-5th) came
out correct, but every other chord had the right scale degrees and the
right hand-comfortable shape, yet the *wrong pitch height* for the notes
`Chunk.reframe` had carried an octave on - e.g. vi's reframed degree-3
(wrapped to degree 1, octave+1) landed a full octave above where the
chord's own 3rd actually belongs, breaking the chunk's own ascending
shape into up-then-down. Ignoring octave didn't just drop a "nice to
have" (a large intentional leap); it dropped information the solver
needed to place *ordinary* reframed notes correctly, since a wrapped
degree's carried octave is exactly what says "this note is genuinely
above the previous ones," not an artifact specific to shared-fixed-anchor
projection.

The real fix keeps octave, but resolves it correctly: every note's own
real, ever-increasing semitone height (`Fretboard.target_pitch_class`'s
pitch class + 12 semitones per octave the `Degree_reference` carries) is
computed *before* any fretboard position is chosen. Reframing shifts
every note in a chunk by the same amount, so two notes a Chunk was
authored an interval apart keep that exact interval after reframing,
wrapped octave or not - this is what makes the harmonic content and the
positional search fully independent rather than needing octave to also
serve as a hand-comfort signal. The first note still resolves via
nearest-occurrence-to-`start_anchor` (searched across a window, same as
before); every later note's *pitch* is now fully pinned by the harmonic
math, and only *which string* reaches it is chosen positionally, within
`max_fret_distance` of wherever the previous note landed. Re-verified the
full I-vi-IV-V run: all four chords now resolve to correct, ascending,
comfortable triads. Two regression tests lock in the specific failure
mode (a reframed wrapped octave, and a deliberately authored octave leap)
so this doesn't silently regress again.

# References

- `worklog/08-chunk-model-design/plan.md` — the design conversation this
  implements.
- `docs/chunk-and-fretboard-model.md` — the model reference this updates.
