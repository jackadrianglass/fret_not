# GUI spike record

Goal: rebuild the previous GUI's fretboard view against a checklist that
stages up from boot to animation, and use it to pick the toolkit. The
outcome: **iced**, and the winning spike was promoted to the real GUI,
`crates/gui` (bin `fret_not`). Toolkit gotchas live in `notes/iced.md`.

## The feature checklist

The spec `crates/gui` implements, in dependency order — each stage built
on the last.

### Stage 0 — boot

Window opens and shows something; build times and the decisions the
toolkit forces up front (event loop shape, state ownership, renderer
backend) noted.

### Stage 1 — static fretboard

The core visual, rebuilt from the previous GUI's feature capture in
`docs/rust-migration-plan.md`:

- One horizontal line per string, light vertical line per fret, fret
  numbers below the lowest string. Fret count comes from `Instrument`,
  never a UI constant.
- Every playable position gets a dot, from `fretboard::degrees_in_window`:
  in-key = filled accent dot with centered degree label, degree-1
  additionally ringed with a halo, off-key = hollow light-gray unlabeled
  circle.
- The custom-drawing test: the fretboard is ~90% bespoke drawing.

### Stage 2 — text and layout

- Control bar above the fretboard: tonic dropdown (12 spelled names),
  quality (Major/Minor), label-mode toggle (Degrees/Notes). These drive
  everything below through `Key`.
- Resizable window; geometry recomputed on resize, nothing anchored to
  hard-coded pixels.
- Text quality checks: dot labels centered inside circles (small text),
  the tonic dropdown (normal text), the title (styled text).

### Stage 3 — interaction

- Changing tonic/quality re-derives the whole board through the core's
  tonic-anchored resolution.
- Hover a dot: it grows/glows.
- Click a dot: selects that degree everywhere it appears (the cheap
  stand-in for the future pick-and-practice flow).

### Stage 4 — chunk and solver view

The data-heavy screen, aimed at the known not-yet-features:

- Slot strip: one cell per slot, degree label or `.` for rests, rests
  dimmed, selected cell with a cursor outline, click to select. `+`/`-`
  buttons append/remove slots.
- Pick-and-practice stub: `chunk_solver::positions` on a hard-coded chunk
  (`[1, b3, 5, 6]` over a I–IV–V progression), prev/next buttons cycle
  the ranked fingerings, the current fingering renders as dots on the
  fretboard and as fret numbers in a simple tab (one line per string,
  open note labels at the left).

### Stage 5 — animation

Four deliberately different kinds, covering the mechanisms a practice
app needs (feedback on state change, time-driven playback, ambient
motion):

- **Key-change bloom.** On tonic change, dots scale+fade in, staggered
  along the neck (leftmost first) — enter/exit transitions on many
  elements at once, with interruption mid-animation.
- **Chunk playhead.** A playhead sweeps the slot strip at a fixed tempo;
  each slot's notes pulse on the fretboard as it's crossed, rests
  skipped — time-driven animation from app logic, running alongside
  interaction.
- **Scale sweep.** All positions of one degree light up in a wave along
  the neck, looping, eased — paint-only animation at 60fps of many small
  moving elements.
- **Hover ease.** The hover growth with a proper ease-out and
  interruption.

All four tick via a `time::every(16ms)` subscription driving `now`
through app state; the first-party `Animation<T>` tween API handles the
easing math (per-element stagger via `delay`).

## Out of scope

- Persistence of any state (still a deliberate core-level deferral).
- Audio, precise tempo timing, MIDI.
- Packaging for web/mobile — the desktop build is the target.
- Accessibility beyond what the toolkit gives for free.
