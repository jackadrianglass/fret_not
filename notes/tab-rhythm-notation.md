# Tab rhythm-row notation — reference

What `crates/gui/src/tab_canvas.rs` draws above the tab staff, and the
standard-notation rules it encodes. Written after the duration-grouping
pass (tab v7); the grid is uniform slots — sixteenths, sixteen per 4/4 bar,
four per beat.

## The model: slots → events → values

The screen hands the canvas a per-column `rests` flag. Within each beat the
canvas parses **events** (`beat_events`):

- A note absorbs the rests that immediately follow it, up to the next note
  or the beat's end. Its **value** is the number of slots it occupies.
- Rests before the beat's first note merge into one leading rest event.
- Absorption never crosses a beat boundary (consequence: the last note in a
  beat always extends to the beat end, and a beat's values always sum to the
  beat's slot count).

Values in a 4-slot beat: `O---` = 4 (quarter), `O-O-` = 2+2 (two eighths),
`OOO-` = 1+1+2 (two sixteenths and an eighth), `O-OO` = 2+1+1, `OO--` = 1+3
(a sixteenth and a dotted eighth), `O--O` = 3+1.

Two helpers derive the notation from a value:

- `beams(value, slots_per_beat)` — the beam level: the largest power of two
  within the value sets it (`(slots_per_beat / base).trailing_zeros()`), so
  1→2 beams, 2→1, 3→1, 4→0 in a 4-slot beat. The same number is the flag
  count on a lone note and the hook count on a rest.
- `dots(value)` — `popcount(value) − 1`; only odd values > 1 are dotted
  (3 = dotted eighth in a 4-slot beat).

## Beaming

Per beat (`beat_beams`), standard metric rules:

- The beat's notes form **one group**. The outer beam (level 0) spans the
  whole group, whatever the values.
- Inner beams (level ≥ 1) connect only notes whose value has that many
  beams — they **break at value changes** (secondary beam breaks).
- A finer note alone at the group's edge carries a **stub**: a short
  fragment (`STUB_LENGTH`) pointing toward the group (right stub for the
  first note, left stub for the last). Interior lone runs cannot occur in
  2- and 4-slot beats (a value-1 note surrounded by value-2 notes needs 5
  slots), so stubs are always edge stubs.
- A beat with **one** note carries flags instead: one per beam level, none
  for a quarter (`O---` renders as a plain quarter — no beam, no flag).
- A beat with no notes is a rest (see below).

## Rests

- Hooks = the rest's beam level (1 hook = eighth rest, 2 = sixteenth,
  3 = 32nd); a dotted rest adds a dot to the right of the glyph.
- A rest filling the beat (`value == slots_per_beat`) is the **quarter
  rest** (`draw_quarter_rest`), drawn larger than the hook family, not part
  of it.

## Glyphs (all line-drawn)

No font glyphs anywhere — `Font::default()` is a platform sans-serif with
no guaranteed musical-symbol coverage, and a missing glyph paints as tofu.

- **Notehead**: a filled oval, `NOTEHEAD_ASPECT`× wider than tall, tilted
  `NOTEHEAD_TILT` (long axis rising to the right). Four bezier arcs with the
  circle constant K = 0.5523 (`notehead_path`); the canvas has no
  rotated-ellipse primitive. The stem attaches at the oval's right extreme
  (`notehead_extent` = the rotated ellipse's horizontal reach).
- **Flag**: a quadratic hook off the stem, concave toward the notehead,
  stacked one per beam level (`FLAG_GAP` apart).
- **Rest (eighth/sixteenth family)**: a near-vertical leg from the top
  right down to the bottom, with one ball-and-arc unit per beam level hooked
  onto it along the slant — each arc emerges from the bottom of its ball and
  sweeps out to its corner on the leg. One hook is the eighth rest, two the
  sixteenth, three the 32nd.
- **Quarter rest**: the outline of a CC0 reference glyph (openclipart "Quarter
  Rest"), stored as absolute bezier segments (`QUARTER_REST_SVG` in
  `tab_canvas.rs`) and mapped onto the staff scaled to the glyph height and
  centered on the column; filled whole, no stroke.
- **Dot**: to the right of the notehead, raised into the space above the
  line (the notehead sits on the one-line staff), clear of the stem.

## Geometry

All design sizes are constants scaled off the font-size setting
(`COLUMN_WIDTH`, `STEM_HEIGHT`, `BEAM_THICKNESS`/`BEAM_SPACING`,
`NOTEHEAD_RADIUS`, `FLAG_*`, `REST_*`, `DOT_*`, `STUB_LENGTH`). Beams stack
down from the stem top, one per level; the outermost straddles the stem's
end. The rhythm row rides `RHYTHM_GAP` above the top string (the detachment
from the tab staff).

## Tests

The planner is pure computation and fully tested inline in
`tab_canvas.rs`: event parsing (the four beat variations, leading rests,
silent beats), `beams`/`dots` per value, and whole-bar plans (groups,
secondary breaks, edge stubs, per-value flags, partial final beats, eighth
grids). Drawing is paint-only and deliberately untested.
