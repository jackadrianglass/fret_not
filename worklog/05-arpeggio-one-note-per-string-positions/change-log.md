# Change log

- Added `lib/theory/arpeggio.ml`/`.mli`: `major`/`minor` triad degree
  filters, mirroring `Pentatonic` exactly (own unexported `keep` helper,
  not shared).
- Fixed a latent bug in `notes_per_string_positions`'s chaining step: it
  indexed into the already-built position (`List.nth_exn this_position 1`)
  to find where the next position starts, which only works when there are
  >= 2 notes on the low string. Replaced with
  `one_step_past_low_string_start`, which searches one more step past the
  low string's own first note directly. Reproduces the old behavior
  exactly for 3nps/2nps (all existing tests still pass unchanged) and is
  now well-defined for 1 note/string.
- Added `Fretboard.arpeggio_degrees`, `arpeggio_positions_in_window`
  (mirroring the pentatonic pair), and `one_note_per_string_positions`
  (mirroring `two_notes_per_string_positions`, plus a `with_closing_note`
  step appending the high string's second note so each position bookends
  back to its own starting tone).
- `bin/main.ml`: Scale dropdown gains "Arpeggio"; Position options become
  "Root/1st Inv/2nd Inv" for it (fixed order, not quality-derived).
- Tests: `Arpeggio.major`/`.minor` degree sets, position count/shape (3
  positions of 7 notes), a hand-verified C major Root position shape,
  the bookend invariant, octave-drop bound, and that
  `arpeggio_positions_in_window` only ever lands on the 3 arpeggio pitch
  classes. 42/42 tests pass.
- Verified via `dune build`, `dune build @fmt --auto-promote`, and
  `dune test` only — THE APP ITSELF WAS NOT LAUNCHED OR VISUALLY CHECKED;
  manual verification of the new Arpeggio scale in the running app is
  still open.
