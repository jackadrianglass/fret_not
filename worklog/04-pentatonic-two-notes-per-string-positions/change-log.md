# Change log

- Generalized `three_notes_per_string_positions`'s walking core (`~count`
  instead of hardcoded 3, position count derived from the diatonic pitch
  class list's length instead of hardcoded 7); behavior unchanged, all
  existing tests still pass.
- Refactored `positions_in_window` to iterate a `Scale_degree.t list`
  directly instead of re-deriving pitch class from a degree number, to
  avoid a `degree - 1` indexing trap on a filtered (non-contiguous) degree
  list.
- Added `Fretboard.two_notes_per_string_positions`,
  `Fretboard.pentatonic_positions_in_window`, and
  `Fretboard.pentatonic_degrees` — the first code wiring
  `lib/theory/pentatonic.ml` into the fretboard/rendering layer.
- `bin/main.ml`: new Scale dropdown (Diatonic/Pentatonic) between Quality
  and Position; Position's options and source list branch on it. Switching
  Scale resets Position to "All" to avoid an out-of-range index.
- Tests: position count/shape, a hand-verified C major pentatonic position
  1 shape, chaining (share 1 of 2 low-string notes), octave-drop bound, and
  that `pentatonic_positions_in_window` only ever lands on the 5 pentatonic
  pitch classes.
- Narrowed `todo.md`'s notes-per-string bullet now that 3 and 2 notes/string
  are both done, leaving only 1-note-per-string open.
- Verified via `dune build`, `dune build @fmt --auto-promote`, and
  `dune test` (35/35 passing) only — THE APP ITSELF WAS NOT LAUNCHED OR
  VISUALLY CHECKED; manual verification of the new Scale/Position controls
  in the running app is still open.
