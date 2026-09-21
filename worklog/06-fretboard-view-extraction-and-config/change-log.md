# Change log

- Added `lib/view/fretboard_view_config.ml`/`.mli`: a config record for
  the fretboard view's visual/layout constants (canvas size, margin,
  fret_count, control-bar geometry, dot/halo radii, font sizes,
  `dimmed_alpha`, spacing gaps, tuning, window title, target FPS), with
  a `default` reproducing the previous hardcoded values exactly.
- Added `lib/view/fretboard_view_state.ml`/`.mli`: today's `state`/
  `label_mode`/`scale` types plus every pure computation function
  formerly in `bin/main.ml` (renamed to drop redundant suffixes:
  `key_of_state` → `key`, `scale_degrees_for` → `scale_degrees`, etc.),
  taking `~config:Fretboard_view_config.t` instead of hardcoding
  `Tuning.standard` or reading a module-level `layout`. Zero raylib
  dependency, fully alcotest-testable.
- Added `lib/layout/row_layout.ml`/`.mli`: `x_positions` (the
  left-to-right control-row cursor math `draw_controls` used to inline)
  and `dropdown_width` (the pure arithmetic half of the old
  `dropdown_width`, split from its raylib-measuring half).
- Added `bin/fretboard_view.ml`: every raylib/raygui-touching function
  (`setup`, `draw_fretboard_grid`, `draw_fret_positions`,
  `draw_in_key_dot`, `draw_off_key_dot`, `draw_centered_text`, the `ui_*`
  style helpers, `dropdown_width`/`label_width`'s raylib-measuring
  wrappers, `draw_controls`, `loop`), parameterized explicitly by
  `Fretboard_view_config.t` and threading `Fretboard_view_state.t` the
  same way `state` was already threaded.
- `bin/main.ml` shrinks to a thin composition root (build config, call
  `Fretboard_view.setup`/`.loop`).
- `docs/chunk-and-fretboard-model.md`: section 8's code map gains the
  4th `lib/` folder (`view/`) and the two new `layout/` /`view/` modules;
  notes that the raylib shell itself lives in `bin/fretboard_view.ml`,
  outside the pure library. Sections 1-7 (the domain model) untouched.
- Tests: `test/layout/row_layout_test.ml` (new) and
  `test/view/fretboard_view_state_test.ml` (new) — `selected_position`
  across all three scales, `position_options` counts/labels,
  `position_label_text`, the `*_of_index` converters, `x_positions`,
  `dropdown_width`'s formula. 54/54 tests pass. No dedicated test for
  `Fretboard_view_config` (a record literal with a default, no logic to
  catch a bug in).
- Verified via `dune build`, `dune build @fmt --auto-promote`, and
  `dune test` only — THE APP ITSELF WAS NOT LAUNCHED OR VISUALLY
  CHECKED; confirming the running app looks and behaves identically
  after the extraction is still open.
