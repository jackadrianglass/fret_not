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
  2. Page content: word editor, tab, sequence, or settings page (whichever
     the page toggle selects)
  3. Fretboard view, pinned to the window bottom regardless of page (a live
     preview of the label-mode and font-size settings on the settings page)

## Control bar

- Left side: page toggle — "edit" / "tab" / "sequence" / "settings" — and the
  word-library rail collapse toggle (both hidden on the settings page).
- Right side, flush to the right edge: tonic dropdown (12 spelled tonic names),
  key quality dropdown (Major / Minor), tempo slider (40–240 bpm) with its
  readout.
- Centered over the bar: playback controls — play/stop button and the current
  step label.
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
  - Off-key positions: faint hollow light-gray rings, unlabeled — the key's
    labeled dots stay the primary display. When a word plays an off-key
    position it emphasizes like any note, labeled with its spelled degree
    ("b3" plus the octave-count subscript) or full SPN ("Eb4").

## Word editing page

- A word is an equal-duration slot list; each slot is a rest or a
  degree-reference note (degree 1–7, alteration b/n/#, octave counted from
  the word's anchor note — its first note — register-free, so words are
  key-independent). Slot presence is the rhythm.
- Slot band: one cell per slot, label is the degree label ("1", "b3", with a
  trailing apostrophe per octave above the anchor note or a trailing comma
  per octave below it) or "." for a rest; rest cells render dimmed, note
  cells in the accent color; every cell carries a quiet border, the selected
  slot an accent border plus an accent underline. Click a cell to select it. Note cells also show the resolved SPN note name
  in the register the word currently sits in.
- Board clicks write the selected slot: every position resolves — in-key
  positions store their degree, off-key positions store their spelled
  accidental (a "b3", a "#4") — with the octave counted from the anchor
  note. Rests are authored by the rest toggle, not by clicks. Each write
  steps the cursor right, stopping at the band's end — growing the word
  is the insert points' and the trailing "+"'s job (bounded by the
  max-slot count). Writing or rewriting the
  anchor note (the word's first note) moves the octave reference, shifting
  the other notes to keep their sounding pitches; removing it re-references
  to the next note. Displayed positions are remembered per word (keyed with
  the session key) and stick until the user touches the slot; a note slot
  never placed from the board fills once from the best current fingering.
  The solve's fret cap starts at the solver's fret-cap setting (settings
  page) and widens to whatever the displayed fingering's widest consecutive
  move requires, so the displayed fingering is always among the solver's
  enumerated fingerings. Changing the key or the instrument drops the
  positions, which refill from the new solve.
- A prev / "i of n" / next stepper under the strip walks the word's rotation
  catalog: every fingering that keeps the root note (the anchor note) pinned
  exactly where the user placed it — string and fret — with the rest of the
  notes enumerated within the word's fret cap around it. The catalog is
  stable while rotating; the cap grows only when the user's own input is
  wider than it admits, never from a swap. A never-placed word's root
  note takes a default starting position — the nearest playable position
  sounding it — and the catalog pins there from the first refresh, so the
  count never collapses after a rotation.
- Each cell carries an "x" button at its top-right corner deleting that
  particular slot; the cursor tracks its slot — a deletion before it shifts
  it left, deleting past the end clamps it onto the new end.
- The band's cells are separated by insert points that rest as faint
  hairline rules; hovering tints a rule to mark it interactive, and
  dwelling on it for a short timer expands it to reveal a "+" inserting
  a rest slot there, so passing over a point on the way to a cell never
  re-flows the band; a trailing "+" compartment (accent, always visible,
  a touch wider) appends one. Both are bounded by the max-slot count,
  and the cursor lands on the fresh slot, so the next board click writes
  into it.
- Inspector row below the strip: Note/Rest toggle, degree dropdown (1–7),
  alteration toggle (b/n/#), octave toggle (0/1). Editing a slot that is a rest
  converts it into a note holding the edited value at its defaults.

### Word variants

- The edit page stacks the edited word's family as slot bands: the root
  word's band first, then one band per variant underneath, all visible at
  once (the stack scrolls vertically when it outgrows the space).
- Each band is labeled at the left — roots lettered in order (A, B, …),
  variants numbered under their root (A1, A2, …) — and is fully editable
  inline: cells, insert points, per-cell delete, exactly like the root's band.
- The edited band (the last one clicked) carries an accent border and an
  accent label; the fretboard, the fingering stepper and the rest toggle all
  follow it. Clicking another band's cell makes that word the edited one and
  moves the cursor onto the clicked slot.
- "+ add variant" at the bottom of the stack appends a variant: a copy of the
  edited word (slots, authored display positions, octave base, solver cap) so
  it opens sounding and looking identical, parented to the root. Lineage is
  flat — a variant never has variants of its own; deriving from a variant
  still parents to the root.
- The word-library rail stays a flat list of every word (roots and
  variants), named by the same lineage; sequence steps reference words by
  index, and variants append at the end so existing references never shift.

## Tab viewing page

- Tab rows: one line per string, labeled at the left with the string's absolute
  open note in SPN (so E2 vs E4 are distinct), note columns showing fret
  numbers, and a thick section-divider rule below.
- Above the staff, a rhythm row notates the word's slots in standard
  notation: a one-line staff with the meter at its left. Within each beat a
  note's value is the number of slots it occupies — it absorbs the rests
  that follow it up to the next note or the beat's end — so O--- is a
  quarter, O-O- two eighths, OOO- two sixteenths and an eighth. Notes beam
  per beat: the outer beam spans the beat's group, inner beams break at
  value changes, and a finer note alone at the group's edge carries a short
  stub. A note alone in its beat is flagged (one flag per beam level, none
  for a quarter), and dotted values carry a dot. Rests merge the same way:
  a hook per beam level, a dot when dotted, and the quarter-rest squiggle
  for a silent beat. Slots are sixteenth notes; a bar is sixteen of them
  (4/4). Glyphs are line-drawn in standard shapes: tilted oval noteheads,
  curved flag hooks, and the ball-and-hooks rest — drawn, not a font glyph,
  since the default font has no guaranteed musical-symbol coverage.
- The staff always shows whole bars: a trailing partial bar is padded with
  rest columns, so every bar is the same size and grid-aligned.
- The data source is the selected word fingering from the solver.

## Settings page

- A whole page (not a popover), reached from the control-bar page toggle. The
  word-library rail is hidden while it is active.
- Rule-divided sections, each a titled group of label + control rows, in a
  scrollable width-capped column:
  - Appearance: theme dropdown (all built-in themes), label-mode dropdown
    (Degrees / Notes), font size number input — the base size every text size
    in the app scales off.
  - Fretboard: tuning dropdown over the known presets (standard 6-string,
    drop D, 7-string, bass 4-string), each shipping its own fret count
    (custom tunings are a follow-up), and the fret cap number input — the
    widest consecutive fret move a fingering may contain.
- Settings live in app state only; nothing persists across restarts yet.

## Known not-yet-features (so experiments can aim at them)

- Reach ranking exists (`Reach.semitone_span`) but no pick-and-practice flow.
- No persistence of any state.
