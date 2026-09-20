# Plan

Add a 3-notes-per-string (3nps) scale sequence view: highlight, for a given
key, one of the 7 classic "mode position" patterns across a compact region
of the neck, so all 7 can be learned and moved between as practice
material.

Not yet settled how the recipe is represented or how a position picks its
fret window — see Questions below. This plan stays narrow: representing
and rendering one position at a time. Sequencing/linking positions into a
full-neck practice run is a later concern, not this pass.

# Changes

All 7 modes of a key share the same 7 pitch classes — only the framing
(which note counts as "degree 1") differs. So the walk itself needs just
the current key/mode's diatonic pitch-class set plus one starting root;
it doesn't need to switch which `Mode.t` is active to visit different
physical dots. "Position 2 (Dorian)" is a label derived after the fact
(which mode's own root the position's first note happens to land on),
not a control the walk depends on. That's smaller than first scoped —
no live 7-way mode selector needed, and today's Major/Minor-only quality
toggle stays as-is.

- New `Fretboard` function: a contiguous per-string walk. Starting from
  an anchor (nearest occurrence of the current key/mode's degree-1 pitch
  class to the low string's open position), take the 3 nearest ascending
  diatonic frets on string 0, then continue the search on string 1
  strictly above the previous string's 3rd note, and so on across all
  strings — one position. The next position's low string starts back at
  its *own* 2nd note (not its 3rd/last), so adjacent positions overlap by
  2 of 3 low-string notes rather than tiling with zero overlap — see the
  "position ordering" question below for why.
- Label each position with the mode whose own root coincides with that
  position's first note (match the note's pitch class against
  `Key.modes`), for display only — reuses the same matching
  `Fretboard.of_position` already does, doesn't change what notes are
  in the position.
- `bin/main.ml`: a Position selector (1-7). When a position is selected,
  restrict rendered dots to that position's subset, full brightness;
  dim (not hide) the rest of the current key/mode's diatonic dots
  elsewhere on the neck, rather than switching to a separate view.
- Dots show note content only (no play-order numbering, no connecting
  lines) — same degree-number/note-name label toggle already in
  `bin/main.ml` applies unchanged.

# Questions

## How should "3 per string" be computed?

Three shapes considered:

**A — window + per-string top-N filter.** Reuse
`Fretboard.positions_in_window` for a fret window, group the results by
`string_index`, keep the lowest 3 frets per string. Sits at the same
layer `positions_in_window` already lives at (concrete, post-projection),
sizing the window is the only new problem to solve.

**B — chained per-string walk.** Don't pre-pick a window at all: start
from an anchor on the lowest string, take its 3 nearest ascending
diatonic frets, then start the next string's search from near where the
previous string's 3rd note landed, repeating across strings. Mirrors the
open "does each chunk note anchor to the previous note, chaining down the
phrase?" question already sitting in
`docs/chunk-and-fretboard-model.md` §7 — same shape of problem, one layer
up.

**C — hardcoded shape templates**, the way `Pentatonic` hardcodes an
allowed-degree list. Doesn't fit as well: pentatonic subsets degrees
*before* projection (abstract layer), but 3nps is inherently about
physical fret spacing (concrete layer) — a hardcoded shape would either
assume standard tuning or need one template per tuning.

### Answer

Chained per-string walk. Walking up the scale (for Ionian, just the major
scale) and finding 3 notes per string that align with it — the notes
land close together on their own, by nature of how the tuning's open
strings are spaced. No pre-sized fret window; each string's 3 notes are
found by walking forward from where the previous string left off.

## What does "position N" mean, and where does it live on the neck?

3nps systems commonly number 7 positions 1-7, each rooted at a degree of
the parent major scale — position 1 = Ionian's pattern, position 2 =
Dorian's, ... position 7 = Locrian's — independent of which
degree the player is calling "the key." That maps onto this codebase's
existing `Mode.t` directly: no new "Position" type needed, just a label
("Position 2 (Dorian)") over the existing 7 modes.

Left open: how each position's anchor/fret-window is chosen (nearest
occurrence of that mode's root to the open string, on the low string?),
and whether "position N" should stay pinned near the nut or be movable up
the neck independently of the others.

### Answer

Think of the first position (Ionian) to be the first major root note on
the low string. Positions 2-7 continue as one contiguous chain up the
neck from there, rather than each being independently anchored to its
own mode's root: position 2 picks up where position 1's walk left off,
position 3 where 2 left off, and so on — one continuous walk up the
neck, tiling it with no gaps or overlaps, until it's spanned about 2
octaves and the pattern repeats.

## How does this interact with the existing whole-neck key/mode overlay?

Options: a toggle that switches the existing view from "whole neck" to
"one position," a second overlay drawn alongside the first (dimmed
out-of-position dots vs. full-brightness in-position ones), or a
completely separate view/screen.

### Answer

Let's dim the unused notes for that particular scale

## What do the dots show — scale degree, or play order?

The existing overlay labels dots with scale-degree number or note name
(toggle already in `bin/main.ml`). A 3nps view could instead (or
additionally) label each string's 3 notes 1st/2nd/3rd to show picking
order, and/or draw connecting lines between consecutive notes the way
tutorial diagrams usually do.

### Answer

Just show the notes used

## Zero-overlap tiling or conventional overlapping order?

Working the note math by hand for C major surfaced a conflict with the
"contiguous chain, no gaps or overlaps" framing agreed above: chaining
each position from the *previous position's last note* does tile the
neck with zero repeated notes, but the 7 positions land on modes in
degree order 1, 4, 7, 3, 6, 2, 5 (Ionian, Lydian, Locrian, Phrygian,
Aeolian, Dorian, Mixolydian) — not the familiar Ionian, Dorian,
Phrygian, Lydian, Mixolydian, Aeolian, Locrian sequence most "7
positions" teaching material uses. The two properties are mutually
exclusive: zero overlap requires each position to start 3 scale-degrees
past the previous one; the conventional sequential mode order requires
starting only 1 scale-degree past — which means adjacent positions
share 2 of their 3 low-string notes and overlap in fret range.

### Answer

Sequential mode order, overlapping. Position N's low string starts one
scale-degree past position (N-1)'s low-string *start* (its 2nd note),
not past its last note — position 2 is Dorian, position 3 is Phrygian,
and so on, matching how guitarists actually learn the 7 positions.

# References

None.
