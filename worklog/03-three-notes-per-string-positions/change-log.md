# Change log

- Add `Key.mode_with_root`: which of a key's 7 modes has its own root on a
  given pitch class.
- Add `Fretboard.three_notes_per_string_positions`: the 7 3nps mode
  positions, chained so position N starts one scale-degree past position
  (N-1)'s own low-string start, landing on modes in sequential order.
- Each position dropped by whole octaves to its lowest playable spot on
  the neck, after position 3 (Phrygian) came back with notes past fret
  17 and off the visible neck.
- `bin/main.ml`: Position dropdown ("All", "1 Ionian", "2 Dorian", ...,
  labels rotated to match the active quality's mode), dims (not hides)
  diatonic dots outside the selected position instead of switching views.
- Tests: `Key.mode_with_root`, position count/shape, the known C major
  Ionian position 1 shape, position 3's octave drop, and that all 7
  positions land on modes in Ionian..Locrian order.
