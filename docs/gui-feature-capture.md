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
  numbers drawn below the lowest string. Strings are drawn in tab order —
  high E at the top — and each row is labeled at the left with its open
  note in SPN (so E2 vs E4 are distinct). Fret count comes from the
  instrument (max fret per string), not from any UI constant.
- Every playable position on the instrument gets a dot:
  - In-key positions: filled dot in the theme accent color, degree-1 positions
    additionally ringed with a black halo, label centered in the dot showing
    either the degree number or the full SPN note name (per label-mode
    dropdown). Degree labels additionally carry the key-referenced octave
    count as a small subscript beside the number — one count per cycle of 7
    degrees above the tonic's register.
  - Off-key positions: hollow light-gray circle, no label.

## Chunk editing page

- A chunk is an equal-duration slot list; each slot is a rest or a
  degree-reference note (degree 1–7, alteration b/n/#, octave counted from
  the chunk's anchor note — its first note — register-free, so chunks are
  key-independent). Slot presence is the rhythm.
- Slot strip: one cell per slot, label is the degree label ("1", "b3", with a
  trailing apostrophe per octave above the anchor note or a trailing comma
  per octave below it) or "." for a rest; rest cells render dimmed, note
  cells in the accent color; the selected slot has a cursor outline. Click a
  cell to select it. Note cells also show the resolved SPN note name in the
  register the chunk currently sits in.
- Board clicks write the selected slot: an in-key position stores its degree
  with the octave counted from the anchor note; an off-key click clears the
  slot to a rest. Each write steps the cursor right, appending a rest at the
  end of the chunk (bounded by the max-slot count). Writing or rewriting the
  anchor note (the chunk's first note) moves the octave reference, shifting
  the other notes to keep their sounding pitches; removing it re-references
  to the next note. Displayed positions are remembered per chunk (keyed with
  the session key) and stick until the user touches the slot; a note slot
  never placed from the board fills once from the best current fingering.
  The solve's fret cap widens to whatever the displayed fingering's widest
  consecutive move requires, so the displayed fingering is always among the
  solver's enumerated fingerings. Changing the key drops the positions,
  which refill from the new key's solve.
- A prev / "i of n" / next stepper under the strip walks the chunk's rotation
  catalog: every fingering that keeps the root note (the anchor note) pinned
  exactly where the user placed it — string and fret — with the rest of the
  notes enumerated within the chunk's fret cap around it. The catalog is
  stable while rotating; the cap grows only when the user's own input is
  wider than it admits, never from a swap. A never-placed chunk enumerates
  around the default window so its slots can fill.
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
