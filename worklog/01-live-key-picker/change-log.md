# Change log

- Added a control bar above the fretboard in `bin/main.ml`: a `dropdown_box` for the tonic and a `toggle_group` for Major/Minor.
- Threaded `{ tonic_index; quality_index; tonic_dropdown_open }` through the render loop; `highlighted_positions` now recomputes each frame from the current `Key.t` instead of a hardcoded C major.
- Dropped the hardcoded C major key, its Ionian assumption, and the window title naming it.
- Fixed the tonic dropdown closing itself the frame after opening: `dropdown_box`'s returned bool is a toggle pulse, not the new edit-mode value — was assigning it directly instead of XORing it against the prior state.
- Documented the dropdown toggle-pulse gotcha in `contributing/raylib-raygui-reference.md` so it isn't rediscovered.
- Fret-number labels were drawn under each grid line (a fret's boundary), not under where that fret's own highlight dot sits (mid-cell). Extracted `Fretboard_layout.fret_center_x` from `position_point`'s x logic and drew the labels there instead, so numbering lines up with the dots.
- Every fret on every string now draws a dot: solid red when the position is in the current key, a hollow gray outline (`draw_circle_lines`) otherwise — previously only in-key positions were drawn at all.
