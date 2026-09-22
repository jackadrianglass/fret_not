# Change log

- Added `lib/chunk/slot.ml`: `Slot.t = Rest | Note of Degree_reference.t`.
- Added `lib/chunk/chunk.ml`: `Chunk.t = Slot.t list` and `Chunk.reframe`,
  reinterpreting a Chunk's own degree numbers against a new root degree
  (mod 7, carrying an octave on wraparound).
- Added `lib/chunk/chord_progression.ml`: `Chord_progression.t = int list`
  of scale-degree roots, plus `apply_to_chunk` producing one reframed Chunk
  per step via `Chunk.reframe`.
- Added `test/chunk/{slot,chunk,chord_progression}_test.ml`, wired into
  `test/test_fret_not.ml`.
- Synced `docs/chunk-and-fretboard-model.md` and `todo.md` to the landed
  model.
- Wired a hardcoded demo Chunk (the 8-note top/5th/3rd/root arpeggio idea)
  reprojected over a `[1;4;5]` `Chord_progression` into `bin/main.ml`,
  replacing the tab viewer's note source so `dune exec bin/main.exe` shows
  a real `Chunk`/`Chord_progression` pipeline output instead of the
  scale-position picker's own notes. Verified all 24 reprojected slots
  resolve to real fretboard positions (no gaps) via a throwaway
  `test/chunk/` check before wiring it in, then removed the check.
- Checked in on the demo's fingering: the shared-anchor projection landed
  on a real but physically awkward voicing (D string fret 10 down to the
  low E string fret 3). Confirmed by hand (two more throwaway checks,
  removed after) that the same abstract chunk also solves cleanly onto
  the existing Locrian 3-notes-per-string position, and that a naive
  chained/nearest-previous-note strategy doesn't reliably find that
  voicing either (it still jumps strings on one note, since fret-number
  distance can't tell a same-string move from a cross-string one).
  Left the demo as-is (both fingerings are equally valid Chunk
  realizations); recorded a modal-position-constrained solver strategy
  and an open question about preserving `Degree_reference.octave`-encoded
  melodic shape through a solver, in both `todo.md` and
  `docs/chunk-and-fretboard-model.md` §8.
- Swapped the demo for a second one exercising what the first didn't: a
  6-slot "OOO-O-"-style riff (root/3rd/5th, Rest, echoed 5th, Rest) over a
  I-vi-IV-V progression. Confirmed by hand (another throwaway check,
  removed after) that every note resolves and that the vi step correctly
  carries an octave when `Chunk.reframe`'s wrap crosses degree 7 (e.g. the
  chunk's own degree-3 slot becomes degree 1, octave+1 under root degree
  6). Surfaced a real gap doing this: `Tab_view` has no way to draw a
  Rest, so `bin/main.ml`'s demo silently drops Rest slots rather than
  showing a gap - the rendered tab plays back denser than the chunk's own
  rhythm. Left as a known, called-out limitation rather than fixed.
- Confirmed a real bug from eyeballing demo #2's actual fret numbers: the
  shared-anchor projection's root-3rd-5th resolved with the 5th *below*
  the root and 3rd (nearest-occurrence-to-one-fixed-anchor doesn't track
  melodic direction). Stripped the demo back to a bare `[1; 3; 5]` triad
  (demo #3, no rests, no progression) and switched `bin/main.ml`'s
  projection to chain each note off wherever the previous one resolved,
  which fixes it for this case (E8 -> A7 -> D5, strictly ascending) -
  confirmed by hand this works because a diatonic triad's own steps are
  always ≤6 semitones, `Fretboard`'s own tie-break window. Also found
  (and did not fix) that continuous chaining across a whole
  Chord_progression compounds badly with `Chunk.reframe`'s octave carry,
  spiraling up the neck after a few chords - recorded as a new open
  question in `todo.md` rather than patched here.
- Added `lib/chunk/chunk_solver.ml`: `Chunk_solver.distance` (raw
  `|fret delta| + |string delta|`) and `Chunk_solver.positions` - a real,
  tested positional solver. Given a starting anchor, a hard per-move fret
  cap, and a sequence of `Degree_reference.t` (already harmonically
  resolved by `Chunk.reframe`), it returns *every* valid way to realize
  them as concrete positions, sorted by total distance, deliberately
  ignoring each degree's own `octave` field (see plan.md's "Positional
  solver design" question for why). Tested in
  `test/chunk/chunk_solver_test.ml` (ascension, the fret cap being
  respected, sort order, an unsatisfiable case returning `[]`, and octave
  being ignored).
- Re-wired `bin/main.ml`'s demo (#4) to solve the full I-vi-IV-V
  progression again, this time correctly: each chord's own reframed triad
  is solved independently via `Chunk_solver.positions` from the same
  `demo_anchor`, taking the lowest-total-distance shape per chord. No more
  cross-chord chaining, no more octave-carry compounding. Confirmed by
  hand (throwaway check, removed after) that all four chords resolve to
  compact, comfortable triads - e.g. vi lands entirely on fret 5 across
  three strings.
- Corrected `Chunk_solver`: ignoring `Degree_reference.octave` (previous
  entry) was wrong, not just a tradeoff - a real I-vi-IV-V run showed every
  non-tonic chord had the right scale degrees but the wrong pitch height
  (e.g. vi's reframed, wrapped-octave 3rd landed a full octave above where
  the chord's own 3rd belongs, breaking the chunk's ascending shape).
  Rewrote `absolute_semitone` to resolve each note's real, ever-increasing
  semitone height (`Fretboard.target_pitch_class` + 12 semitones per
  octave the degree carries) *before* choosing any position - reframing
  shifts every note by the same amount, so this exactly preserves whatever
  interval the chunk was authored with. Only *which string* reaches that
  already-determined pitch is now chosen positionally. Exported
  `Fretboard.target_pitch_class` (previously private) to support this.
  Replaced the "octave is ignored" test with two regression tests: a
  reframed wrapped-octave case (must land a real minor third, not a
  wrapped-octave-inflated one) and a deliberately authored octave leap
  (must land exactly 12 semitones). Re-verified the full I-vi-IV-V run by
  hand: all four chords now resolve to correct, ascending, comfortable
  triads.
