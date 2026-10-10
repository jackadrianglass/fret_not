# GUI feature capture

What the app lets the user do, described toolkit-agnostically so the GUI can
be rebuilt against any Rust GUI toolkit. `crates/gui` (iced) implements this;
`docs/gui-spikes.md` records the staged checklist it was built against.

## App shell

- Single window, target-FPS immediate-mode loop: state is immutable and
  threaded through the frame; each frame recomputes geometry (resize-safe).
- Resizable window, starts maximized, with a minimum size derived from content
  layout. Width fills the screen; content is anchored by rows.
- Vertical stack, top to bottom:
  1. Control bar (fixed height, tinted background, rule along its bottom edge)
  2. Page content: chunk editor page or tab page (whichever the page toggle
     selects)
  3. Fretboard view, pinned to the window bottom regardless of page

## Control bar

- Left side: page toggle — "Edit" / "View" (chunk editing / tab viewing).
- Right side, flush to the right edge: tonic dropdown (12 spelled tonic names),
  key quality dropdown (Major / Minor), static "showing" label, label-mode
  dropdown (Degrees / Notes).
- Dropdowns open and close on click; open state is part of the UI state.
- Tonic + quality together determine the key and mode used by everything below.

## Fretboard view (always visible, bottom)

- Grid: one horizontal line per string, light vertical line per fret, fret
  numbers drawn below the lowest string. Fret count comes from the instrument
  (max fret per string), not from any UI constant.
- Every playable position on the instrument gets a dot:
  - In-key positions: filled dot in the theme accent color, degree-1 positions
    additionally ringed with a black halo, label centered in the dot showing
    either the degree number or the note name (per label-mode dropdown).
  - Off-key positions: hollow light-gray circle, no label.

## Chunk editing page

- A chunk is an equal-duration slot list; each slot is a rest or a
  degree-reference note (degree 1–7, alteration b/n/#, octave 0 or 1). Slot
  presence is the rhythm.
- Slot strip: one cell per slot, label is the degree label ("1", "b3", with a
  trailing apostrophe per octave above the tonic) or "." for a rest; rest cells
  render dimmed, note cells in the accent color; the selected slot has a
  cursor outline. Click a cell to select it.
- "+" button after the last cell appends a rest (bounded by a max-slot count);
  "-" removes the selected slot (cursor moves left, or to 0 at the left edge).
- Inspector row below the strip: Note/Rest toggle, degree dropdown (1–7),
  alteration toggle (b/n/#), octave toggle (0/1). Editing a slot that is a rest
  converts it into a note holding the edited value at its defaults.

## Tab viewing page

- Tab rows: one line per string, labeled at the left with the string's absolute
  open note in SPN (so E2 vs E4 are distinct), note columns showing fret
  numbers, and a thick section-divider rule below.
- The intended data source is the selected chunk fingering from the solver.

## Known not-yet-features (so experiments can aim at them)

- Reach ranking exists (`Reach.semitone_span`) but no pick-and-practice flow.
- No persistence of any state.
