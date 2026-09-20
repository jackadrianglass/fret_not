# Plan

Close out the last two open items from foundations before starting on the fretboard viewer:

1. Replace the guessed octave range (`[-1;0;1;2;3;4]`) used to highlight a key/mode across the whole neck with a computed bound, and give `Fretboard` a function that returns every matching position in a fret window directly, instead of the caller looping over degrees/octaves and filtering by hand.
2. Represent major and minor pentatonic as subsets of `Mode.degrees Ionian`/`Aeolian`, rather than a separate interval formula.

# Changes

- Add `Fretboard.positions_in_window` — computes an exact per-degree octave bound from the tuning and fret window, instead of guessing one.
- Test it against the existing guessed-range search for C major / standard tuning / frets 0-12, to confirm it produces the same result set.
- Wire `bin/main.ml`'s demo to use it, dropping the `generous_octave_range` hack.
- Add `Pentatonic.major` / `Pentatonic.minor`, filtering `Mode.degrees` through an unexported per-module helper.
- Test both presets against their known pitch classes.

# Questions

## How much should this pass bundle together?

The ask to start was "work on the fretboard visualization and get it in a decent spot." The obvious next task looked like the full-neck key+mode overlay, but that only made sense once a few things were settled: how much to bundle (correctness only, vs. also labeling, vs. also live Key/Mode/Tuning switching), and whether to fix the octave-range gap in the same pass.

### Answer

Foundations turned out not to be finished — pentatonic subsets and the bounded region search were still open, which reframed the question entirely. Redirected to close those out first; the fretboard-viewer work itself is a separate, later thread (now sitting in `todo.md`).

## Is pentatonic's degree-subset mechanism the same one exercise-subsets will want?

The pentatonic task's own done-when left this open on purpose, deferring the decision until the exercise-subsets recipe model's shape was clearer.

### Answer

Checked directly rather than deferring further: no. The exercise-subsets recipe model is about *position-level* recipes (notes-per-string, fret-range, string-range — filtering fretboard positions after projection). Pentatonic is *degree-level* (dropping the 4th/7th from `Mode.degrees` before projection ever happens) — a different layer. Kept it minimal: two named presets over an unexported filter helper, no generalized mechanism exposed. The reuse question itself got moved into `todo.md` under Exercise Subsets rather than resolved or dropped.

# References

None.
