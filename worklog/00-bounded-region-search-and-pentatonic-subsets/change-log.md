# Change log

- Added `Fretboard.positions_in_window`, computing an exact per-degree octave bound from the tuning and fret window instead of a guessed range.
- Added a test comparing `positions_in_window` against the existing guessed-range reference search for C major / standard tuning / frets 0-12.
- Wired `bin/main.ml`'s demo to `positions_in_window`, removing the `generous_octave_range` hack and its explanatory comment.
- Added `Pentatonic.major` and `Pentatonic.minor`, filtering `Mode.degrees Ionian`/`Aeolian` through an unexported helper.
- Added tests for both pentatonic presets, registered in the test runner.
- `dune build @fmt --auto-promote` and full test suite passing (21 tests).
