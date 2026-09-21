# Change log

- Added `lib/layout/tab_layout.ml`/`.mli`: pure tab-area geometry
  (`canvas_height`, `string_y`, `note_x`), mirroring `Fretboard_layout`.
- Added `lib/view/tab_view_config.ml`/`.mli`: margin, row spacing, font
  sizes, rule gap, with a `default`.
- Added `bin/tab_view.ml`: `draw` renders labeled string lines, fret
  numbers for the current position's notes (none when Position is "All"),
  and the delineating rule; returns the fretboard's new `top_y`.
- `bin/fretboard_view.ml`: moved `Raylib.init_window`/`set_target_fps` out
  of `setup`; replaced `below_control_bar` with `offset_y ~top_y`, threaded
  through `draw_fretboard_grid`/`draw_fret_positions`; removed `loop`.
- `bin/main.ml`: now the composition root — computes total window height
  from both configs, owns window init and the render loop, wires
  `Tab_view.draw`'s returned `top_y` into the fretboard drawing calls.
- Tests: `test/layout/tab_layout_test.ml` (new) — `canvas_height`,
  `string_y`, `note_x` against hand-picked values. 59/59 tests pass.
- Verified via `dune build`, `dune build @fmt --auto-promote`, and
  `dune test` only — THE APP ITSELF WAS NOT LAUNCHED OR VISUALLY CHECKED;
  seeing how the stacked tab+fretboard layout actually looks (the explicit
  point of this first cut) is still open, and where the next round of
  iteration starts.
- Feedback from running it: the delineating rule read as a 7th string
  (same margin-to-margin span and hairline weight as the tab's string
  lines). Fixed: the rule now spans the full canvas width and draws as a
  thick filled bar (`Tab_view_config.rule_thickness`, default 3px, black)
  instead of a 1px line, so it reads as a section divider.
- Feedback: make the fretboard visualization more compact vertically.
  `Fretboard_view_config.canvas_height` (400 -> 280) is the only lever
  needed — it's independent of `canvas_width`/`margin`'s effect on fret
  spacing, so string spacing tightens (60px -> 36px) with zero change to
  the horizontal layout.
- Full-screen window, current content centered: threaded an explicit
  `(offset_x, offset_y)` through `Fretboard_view.draw_fretboard_grid`/
  `draw_fret_positions`/`draw_controls` and `Tab_view.draw` (and its
  internal helpers) — deliberately NOT via a `Raylib.Camera2D`/
  `begin_mode_2d` transform, since raygui's click hit-testing reads raw
  screen-space mouse coordinates and isn't camera-aware, which would have
  desynced clicks from the visually-shifted control bar dropdowns.
- First attempt at the window sizing itself was broken (blank screen, no
  visible content): it queried `get_monitor_width`/`get_monitor_height`
  (the monitor's *pixel* resolution) and fed that straight into
  `set_window_size` (which expects *points*) — those differ on a HiDPI
  display, so the window ended up sized in a different coordinate system
  than every draw call uses, pushing the centered content way outside the
  visible area. Fixed by using `Raylib.toggle_borderless_windowed ()`
  instead, raylib's own dedicated call for this exact case ("resizes
  window to match monitor resolution") — no manual monitor-pixel math at
  all. `offset_x`/`offset_y` are recomputed every frame from
  `get_screen_width`/`get_screen_height` (rather than once before the
  loop) since the resize may only become current after a frame or two of
  event polling.
- Feedback: make the tab viewer and fretboard a little bigger now that
  there's full-screen room to grow into. Scaled `Fretboard_view_config`
  and `Tab_view_config`'s defaults together by roughly 1.3x (canvas
  dimensions, margins, dot/halo radii, font sizes, tab row spacing/rule
  sizing) — control-bar-specific fields left untouched since only the
  tab/fretboard views were asked about.
