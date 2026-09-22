# Conversation

This entry turns `worklog/08-chunk-model-design/`'s Model sketch into code.
That design conversation had already settled the shape — `Slot.t`,
`Chunk.t`, `Chord_progression.t`, and how a Chunk reprojects over a
progression step — so this pass skipped straight to implementation rather
than re-running the plan/question loop.

`lib/chunk/` landed as a new folder alongside `theory/`, `fretboard/`,
`layout/`, and `view/`, holding three small modules: `Slot.t = Rest | Note
of Degree_reference.t`, `Chunk.t = Slot.t list` with `Chunk.reframe`, and
`Chord_progression.t = int list` with `apply_to_chunk`. None of them store
a Key or Mode on themselves, matching how `Degree_reference.t` already
stays meaningless until paired with one at projection time.

The one real design call made during implementation (rather than left for
review) was whether reframing a Chunk's degree numbers onto a new
progression-step root needed to carry an octave when the arithmetic wraps
past 7. The design doc's own phrasing ("wrapping mod 7") didn't say either
way, but a bare relabel would silently under-place a wrapped note by a full
octave once it's actually projected through `Fretboard` — `octave` there is
a real ±12-semitone offset, not just a display label. `Chunk.reframe`
carries the octave; `worklog/09-chunk-library-core/plan.md`'s Questions
section records the reasoning in place of a comment in the code.

`docs/chunk-and-fretboard-model.md` gained a new §7 for this layer, with
the old open-threads section renumbered and reworded now that the
multi-note-anchoring question is reframed as the still-open
Chunk-to-Fretboard Solver work rather than a live unknown blocking Chunk
itself. `todo.md`'s Chunk Library and Chord Progressions sections got their
landed bullets struck through in place, not deleted, since both sections
still have real work left (library browser, tagging, persistence, the
solver itself).

All 70 existing and new tests pass; `dune build @fmt --auto-promote` is
clean. No UI or persistence work happened here — that's the explicit
next step, to be scoped once there's agreement on how to exercise this
from the app.
