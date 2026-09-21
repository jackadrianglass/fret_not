# Plan

`bin/main.ml` grew to cover three tangled concerns: pure UI-state
computation (which scale/position is selected, what label to draw), a pile
of hardcoded presentation constants (canvas size, dot radii, spacing,
alpha), and the actual raylib/raygui drawing calls. Now that three scale
overlays exist (3nps, 2nps pentatonic, arpeggio) and more GUI elements are
coming, split these apart so each can grow independently.

`lib/layout/fretboard_layout.ml` is the existing pattern to build on: a
plain, pure, tested record with zero raylib dependency. `lib/dune` confirms
`lib/fret_not` depends only on `base`; `bin/dune`'s executable is the only
place `raylib`/`raygui` are linked. That boundary is worth preserving.

# Changes

## `lib/view/` (new, pure, no raylib)

- `fretboard_view_config.ml`/`.mli` — a config record for the
  visual/layout constants (canvas size, margin, fret_count, control-bar
  geometry, dot/halo radii, font sizes, `dimmed_alpha`, spacing gaps,
  tuning, window title, target FPS) with a `default` reproducing today's
  exact hardcoded values. Not in scope: dropdown vocabulary strings
  ("C;C#;D;...", "Major;Minor", "showing") — fixed domain vocabulary, not
  tunable presentation.
- `fretboard_view_state.ml`/`.mli` — today's `state`/`label_mode`/`scale`
  types plus every pure `_for`/`of_index` function from `bin/main.ml`,
  renamed to drop redundant suffixes now that they live in their own
  module. Takes `config:Fretboard_view_config.t` instead of hardcoding
  `Tuning.standard`/reading a module-level `layout`.

## `lib/layout/row_layout.ml`/`.mli` (new)

A second, genuinely new extraction: the left-to-right x-cursor math
`draw_controls` inlined (`tonic_x`, `quality_x`, ... each `= previous_x +.
previous_width +. control_gap`) becomes `Row_layout.x_positions`, pure and
tested. `dropdown_width`'s formula (`left_padding + widest_option_width +
arrow_padding + gap`) similarly splits into `Row_layout.dropdown_width`
(pure) plus a thin raylib-measuring wrapper in `bin/fretboard_view.ml`.

## `bin/fretboard_view.ml` (new) — the raylib shell

Everything touching `Raylib`/`Raygui` moves here, parameterized explicitly
by `Fretboard_view_config.t`, threading `Fretboard_view_state.t` the same
way `state` was already threaded (no module-level mutable globals):
`setup`, `draw_fretboard_grid`, `draw_fret_positions`, `draw_in_key_dot`,
`draw_off_key_dot`, `draw_centered_text`, the `ui_*` style/measurement
helpers, `dropdown_width`/`label_width`, `draw_controls`, `loop`.

## `bin/main.ml`

Shrinks to a thin composition root: build `Fretboard_view_config.default`,
call `Fretboard_view.setup`/`Fretboard_view.loop` with
`Fretboard_view_state.initial`.

## `docs/chunk-and-fretboard-model.md`

Section 8's code map gains the 4th `lib/` folder (`view/`, "pure UI
state/config, still no raylib") and `Row_layout`. No changes to sections
1-7 — the domain model itself is untouched.

# References

None.
