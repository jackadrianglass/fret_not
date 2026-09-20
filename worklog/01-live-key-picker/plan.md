# Plan

Pick up the "Fretboard Viewer" thread from `todo.md`: let a Major/Minor
key be chosen live in the running app and see that key's own 7 diatonic
notes highlighted across the whole neck, instead of the hardcoded C major
/ Ionian / standard tuning demo `bin/main.ml` currently draws. Mode stays
a separate, later thread — quality alone picks the mode used internally
(Ionian for Major, Aeolian for Minor) just to get the right note set, not
exposed as its own control.

# Changes

- Add a control bar above the fretboard canvas in `bin/main.ml`: a
  `dropdown_box` for the tonic (12 pitch classes) and a `toggle_group` for
  Major/Minor quality.
- Thread `{ tonic_index; quality_index; tonic_dropdown_open }` through the
  render loop as state (per the raylib program shape — widgets return
  their new value each frame, no mutation).
- Recompute the `Key.t`, mode (Ionian/Aeolian following quality), and
  `highlighted_positions` each frame from that state (tuning fixed at
  `Tuning.standard`, anchor fixed at open low E, per the Questions below).
- Drop the hardcoded `Key.create ~tonic:Pitch_class.c ~quality:Major` and
  the window title string that names it explicitly.

# Questions

## Does this pass also expose a Mode selector, or does mode follow quality?

`Key.t` is tonic + Major/Minor only — the 7 diatonic modes are a separate
axis (`Mode.t`), and `todo.md` lists "Tuning/Key/Mode switch UI" as one
bundled item under Fretboard Viewer. The ask here was specifically "pick a
key," which reads narrower than that bundled item.

### Answer

Modes stay a separate thread. This pass is just Major/Minor key selection
showing that key's own 7 diatonic notes — mode follows quality (Ionian for
Major, Aeolian for Minor), not independently pickable.

## Tuning: fixed at standard for this pass, or also switchable now?

Same bundling question as above, for the tuning axis. Keeping it fixed
keeps this pass narrowly about the key.

### Answer

Fixed at standard tuning for this pass.

## Control style for picking the tonic (12 options) and quality (2 options)?

This is a "how it looks/feels" call. Options considered:
- `toggle_group` for both — all 12 tonics visible as buttons at once, plus
  a 2-way Major/Minor toggle.
- `dropdown_box` for the tonic (compact, one open at a time) plus a 2-way
  `toggle_group` for quality.

### Answer

`dropdown_box` for the tonic, `toggle_group` for Major/Minor quality.

# References

None.
