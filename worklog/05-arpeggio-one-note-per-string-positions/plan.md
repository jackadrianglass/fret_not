# Plan

Add a third overlay alongside 3nps
(`worklog/03-three-notes-per-string-positions/`) and 2nps pentatonic
(`worklog/04-pentatonic-two-notes-per-string-positions/`): major/minor
triad arpeggios walked 1 note per string across the neck.

A triad has only 3 pitch classes (root/3rd/5th), so strict
1-note-per-string across 6 strings cycles the triad exactly twice (6
notes). Confirmed this is the intended shape, with one addition: each
position's high string carries a second note that continues the same
ascending walk one more step, so every position bookends back to its own
starting tone (Root position: 7 notes, starts and ends on the root; 1st
Inversion: starts and ends on the 3rd; 2nd Inversion: starts and ends on
the 5th).

While re-reading the walking core generalized for 2nps
(`notes_per_string_positions` in `lib/fretboard/fretboard.ml`), found a
latent bug this feature exposes: the chaining step that decides where the
next position starts (`List.nth_exn this_position 1`) implicitly assumes
at least 2 notes per string — for `notes_per_string = 1` there is no index
1 on the low string. Fixed as part of this work, not worked around.

# Changes

## `lib/theory/arpeggio.ml` / `.mli` (new)

Mirrors `lib/theory/pentatonic.ml` exactly: `major ~root = keep
(Mode.degrees Ionian ~root) ~allowed:[ 1; 3; 5 ]`, `minor ~root = keep
(Mode.degrees Aeolian ~root) ~allowed:[ 1; 3; 5 ]`, own unexported `keep`
helper (not shared with `Pentatonic`'s copy) — matches the project's
established precedent of not generalizing this until a real "recipe
model" exists.

## `lib/fretboard/fretboard.ml` / `.mli`

- Correctness fix: replaced the `List.nth_exn this_position 1` chaining
  step with `one_step_past_low_string_start`, which searches one more
  step past the low string's own first note directly rather than indexing
  into the already-built position. Reproduces the old behavior exactly
  for `notes_per_string >= 2` (regression-checked via the existing
  3nps/2nps tests), and is now well-defined for `notes_per_string = 1`.
- Added `arpeggio_degrees`, `arpeggio_positions_in_window` (mirroring the
  pentatonic pair) and `one_note_per_string_positions` (mirroring
  `two_notes_per_string_positions`, plus a `with_closing_note` step
  appending the high string's second note after the base walk).

## `bin/main.ml`

- `scale` gains an `Arpeggio` case; Scale dropdown becomes
  "Diatonic;Pentatonic;Arpeggio".
- `highlighted_positions_for`/`scale_degrees_for` get an `Arpeggio` arm
  sourcing from the new `Fretboard` functions — same reconciliation as
  pentatonic: the 4 dropped diatonic degrees fall out of scope entirely
  rather than being dimmed.
- `one_note_per_string_positions_for` (mirrors
  `two_notes_per_string_positions_for`); `selected_position_for` and
  `position_options_for` get a third branch. Arpeggio positions are
  labeled "Root", "1st Inv", "2nd Inv" (fixed order).
- No changes needed to the scale-switch position-reset logic or to
  `draw_fret_positions` — both already generic.

# References

None.
