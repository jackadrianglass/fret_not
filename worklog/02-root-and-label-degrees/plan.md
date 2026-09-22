# Plan

The fretboard viewer currently shows *which* frets are in the selected
key (solid red dot) versus not (hollow gray ring), but says nothing about
*what* each dot is. This picks up `todo.md`'s "Root and degree labeling"
item: label each in-key dot with either its scale-degree number or its
literal note name, toggle between the two live, and make the root note
visually obvious at a glance.

All theory-side pieces already exist and are public — this is
`bin/main.ml`-only, no new library API:

- `Mode.degrees` (`lib/theory/mode.mli`) + `Key.mode_root`
  (`lib/theory/key.mli`) together give the 7 `{ degree; pitch_class }`
  pairs for the current key/mode — the same call `Fretboard.mode_degrees`
  already makes internally (`lib/fretboard/fretboard.ml:3`).
- Matching a drawn position's concrete pitch class
  (`Tuning.pitch_class_at`) against that list gives the `Scale_degree.t`
  to label it with. `degree = 1` is always the root, since mode-follows-
  quality means the mode's own root and the key's tonic coincide by
  construction.
- Off-key (hollow) dots are unaffected — no label, no root check.

# Changes

- `state` gains `label_mode_index : int`, driven by a new `toggle_group`
  ("Degrees\nNotes") in the control bar; converted to a real
  `label_mode = Degree_number | Note_name` variant, mirroring how
  `quality_of_index` already converts its raw toggle_group int.
- `draw_fret_positions` labels each in-key dot with its degree number or
  note name (per `label_mode`), centered via `Raylib.measure_text`, white
  text on the red fill.
- Root notes (`degree = 1`) additionally get a halo: an outline ring
  drawn around the filled circle in a contrasting color.
- Bump `position_dot_radius` from 8 to ~10-11px so a 1-2 character label
  stays legible (fret spacing ~92px, string spacing 60px — plenty of
  headroom).
- Control bar: new label-mode toggle inserted between the quality toggle
  and the tonic dropdown, keeping the dropdown last (must stay last so
  its expanded list overlays correctly).

# Questions

## How should the root note stand out from the other in-key dots?

Options considered: a halo ring around the same red fill, a distinct
fill color (e.g. gold) for the root only, or a square/rounded-rect shape
instead of a circle for the root only.

### Answer

Halo ring — same red filled circle as every other in-key degree, plus a
second outline ring drawn around it in a contrasting color.

# References

None.
