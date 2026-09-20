# Plan

Add a pentatonic 2-notes-per-string (2nps) scale visualization alongside
the existing 3-notes-per-string (3nps) one from
`worklog/03-three-notes-per-string-positions/`. `lib/theory/pentatonic.ml`
already computes major/minor pentatonic as degree-level subsets of
`Mode.degrees`, but it's wired into nothing — no `Fretboard` code, no UI.
This is the first time pentatonic reaches the fretboard/rendering layer.

# Changes

## UI shape

Two-axis: a new "Scale" dropdown (Diatonic / Pentatonic) sits next to the
existing "Position" dropdown; Position's options recompute based on Scale
(today's "All, 1 Ionian..7 Locrian" for Diatonic; a new "All, 1..5" for
Pentatonic). Control bar order:
`[Tonic][Quality][Scale][Position][showing][Degrees/Notes]`.

## Implementation reuse

Generalize the shared walking/window-search code internally, but keep two
named public entry points — one for 3 notes/string, one for 2 — rather
than one fully generic function or two fully duplicated ones.

- `three_ascending_diatonic_positions` and
  `one_three_notes_per_string_position` become `~count`/
  `~notes_per_string`-parameterized. The chaining index that decides where
  the next position's low string starts (`List.nth_exn this_position 1`)
  stays literally unchanged: index 1 is "one scale-step past the start"
  regardless of count — for 2 notes/string it's simply the last note,
  giving the clean analog of 3nps's "shares 2 of 3" (2nps positions share
  1 of 2 low-string notes).
- The outer recursive walk's position count (hardcoded `7` today) becomes
  `List.length diatonic_pitch_classes` — walking one scale-degree per
  position always covers exactly one octave of whichever scale is in
  play, so the count doesn't need to be passed separately.
- `Fretboard.two_notes_per_string_positions : key:Key.t -> tuning:Tuning.t
  -> Fretboard_position.t list list` — no `~mode` parameter, since
  pentatonic-major/minor aren't independently selectable the way the 7
  diatonic modes are. Picks major/minor pentatonic from `Key.quality key`,
  rooted at `Key.tonic key` directly (verified equal to `Key.mode_root`
  on the quality's own mode in both cases).
- `Fretboard.pentatonic_positions_in_window` — a pentatonic sibling of
  `positions_in_window`, needed for the "Pentatonic + All" whole-neck
  view. Refactored the shared window-search to iterate a
  `Scale_degree.t list` directly (via each entry's own `.pitch_class`)
  instead of re-deriving pitch class from a degree number — the existing
  `target_pitch_class`'s `List.nth_exn (mode_degrees ~key ~mode) (dr.degree
  - 1)` trick relies on the list being a contiguous 1..7, which a
  pentatonic-filtered 5-element list isn't.

## Rendering (`bin/main.ml`)

- New `scale_index`/`scale_dropdown_open` state, mirroring the existing
  per-dropdown shape. Switching Scale resets `position_index` to "All",
  since Diatonic/Pentatonic have different-length position lists.
- `highlighted_positions_for` and `scale_degrees_for` branch on
  `scale_index` to source from the diatonic or pentatonic functions.
  Pentatonic's dropped degrees fall out of scope entirely (rendered
  off-key) rather than being a dimmed subset of the diatonic view — that's
  what makes "Pentatonic + All" a real whole-neck pentatonic overlay.
- `selected_position_for`/`position_options_for` branch the same way to
  pick from `three_notes_per_string_positions_for` /
  `two_notes_per_string_positions_for`, and to size the Position dropdown
  (8 entries vs. 6).
- `draw_fret_positions` needs no changes — its inputs already fully
  determine what's drawn; the dim rule ("dim what's outside the selected
  position, among already in-key notes") applies unchanged to both scales,
  since Pentatonic's in-key set already excludes the 2 dropped degrees.

## Labels

Pentatonic positions labeled plain "1".."5" — no established convention
to lean on the way 3nps has Ionian..Locrian.

# References

None.
