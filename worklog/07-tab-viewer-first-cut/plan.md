# Plan

First cut of the tab viewer from `todo.md`'s "Tab Editor Core" backlog, but
deliberately narrower: just render the note sequence for whichever scale
position is currently selected — plain fret numbers on string lines, no
rhythmic notation — stacked above the existing fretboard viewer, so the
look can be iterated on next. Reads off the exact same selection driving
the fretboard below it (one shared `Fretboard_view_state.t`), not an
independent selector.

`Fretboard_view_state.selected_position` (from
`worklog/06-fretboard-view-extraction-and-config/`) already returns
exactly the ordered `Fretboard_position.t list option` a tab needs —
positions are already stored in play order (string 0 ascending, then
string 1, ...) — so no reordering logic was needed, only rendering.

# Changes

- Added `lib/layout/tab_layout.ml`/`.mli`: pure layout geometry mirroring
  `Fretboard_layout`, but deriving canvas height *from* row spacing
  (`canvas_height`) rather than fitting into a fixed box, since the tab
  area is exactly as tall as its string rows need.
- Added `lib/view/tab_view_config.ml`/`.mli`: margin, row spacing, font
  sizes, rule gap. No `canvas_width`/`tuning` fields — those come from
  `Fretboard_view_config` at composition time so the two views can't drift
  out of sync on dimensions they share.
- No new `lib/view/tab_view_state.ml` — the only computation turning
  `selected_position`'s result into drawable notes is a plain `List.mapi`,
  thin enough to live directly in the rendering code.
- Added `bin/tab_view.ml`: draws labeled string lines (open-string note
  name), fret-number text per note when a position is selected, and the
  delineating rule. When Position is "All" (`notes = None`), the string
  lines still draw (fixed reserved height, no fretboard jump) but no fret
  numbers do.
- `bin/fretboard_view.ml`: `setup` no longer calls `Raylib.init_window`/
  `set_target_fps` — window creation is a whole-app concern now that two
  views share one window, not one view's. `below_control_bar` became
  `offset_y`, taking `~top_y` as an explicit parameter instead of deriving
  it from `config.control_bar_height` alone; `draw_fretboard_grid` and
  `draw_fret_positions` both gained a `~top_y` parameter. `loop` moved out
  entirely.
- `bin/main.ml` is now the composition root: computes total window height
  from both configs, owns `Raylib.init_window`/`set_target_fps`, and the
  render loop — each frame calls `Tab_view.draw` (getting back the y where
  the fretboard should start), then the fretboard drawing functions with
  that `top_y`.

# References

None.
