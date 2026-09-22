# Change log

No code changed — this entry is a design conversation, recorded via
`plan.md`'s Context/Questions/Model sketch, not an implementation diff.

- Settled: a Chunk is `Slot.t list` where `Slot.t = Rest | Note of
  Degree_reference.t` — no separate rhythm representation, every slot
  equal duration, "apply over a subdivision" means picking how many real
  time-grid units the slot list spans (enables deliberate polyrhythms,
  e.g. 4 slots over a 3-grouping).
- Settled: a Chord Progression is its own movable object (`int list` of
  scale-degree roots), not a Song attribute — quality inferred
  diatonically for now, no per-step override. Reprojecting a Chunk over a
  progression step reuses the same rotate-by-N-degrees move
  `Mode.degrees` already makes for reframing a mode's own root.
- Settled: variant lineage is a strict tree, not a DAG.
- Deliberately still open: the chunk-to-fretboard "solver" (multiple
  candidate fingerings per chunk, at least a shared-anchor and a chained/
  walking strategy) is named but not designed in detail yet. Whether a
  chunk-applied-to-a-progression is worth persisting on its own, and
  persistence generally, are both deferred until there's a model worth
  using day to day.
- Surfaced (not resolved): `todo.md`'s pre-existing Tab Editor Core
  "Rhythm Cell" (real, possibly-uneven rhythm for full tab transcription)
  and this conversation's new `Slot.t` (deliberately simpler, equal-
  duration only) are two different rhythm representations with an
  unresolved relationship — flagged as an open question rather than
  silently conflated.
- `todo.md` restructured: Chunk Library's entity model updated to
  Slot-based; new "Chord Progressions" and "Chunk-to-Fretboard Solvers"
  sections added; Permutation Engine and Songs & Practice Context updated
  to match (progression moved out from under Song).
