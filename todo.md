# Todo

Ideas discussed but not started. Grouped loosely by theme for scanability — no priority or dependency implied by order or grouping. When something here becomes the active thread, it graduates to a `worklog/` entry and gets dropped from here once it actually lands.

## Tab Editor Core

_Viewing and editing a tab with real rhythmic notation, not just fret numbers — stops short of chunk extraction or import/export._

- Tab event model: a Fretboard Position + Rhythm Cell + optional articulation (slide, bend, hammer-on), grouped into measures under a time signature. Format-agnostic — no import/export assumptions baked in.
- Open question — Rhythm Cell vs. Chunk Slot: this Rhythm Cell is for real, possibly-uneven rhythm transcribed from an actual tab; the Chunk Library's `Slot.t` (see below) is deliberately simpler — equal-duration slots only, rhythm expressed purely as which slots are rests. Whether/how these two rhythm representations relate (or Tab-to-Chunk Extraction needs a lossy conversion between them) is unresolved — surfaced during `worklog/08-chunk-model-design/`'s chunk-model conversation, not decided there.
- Tab entry/editing UI: place, move, delete, and re-duration notes directly on strings/frets; add/remove measures.
- Rhythm notation rendering: stems/beams/durations rendered alongside the tab, staying aligned with it; rests representable.
- Tab cursor/scrubbing: a visual-only cursor stepping through Tab Events — no audio.
- Tab region selection: select a contiguous measure range, highlighted, readable back as an ordered list of Tab Events. Feeds tab-to-chunk extraction later.
- Open question — future import/export constraints: Ultimate Guitar text/Guitar Pro/MusicXML import/export is deliberately deferred; write down the constraints the Tab Event model must satisfy now (e.g. articulation optional, rhythm not grid-locked) so adding an importer later doesn't force a rework. No importer gets built as part of this.

## Fretboard Viewer

_Seeing a key or mode laid out across the whole neck, with the controls that make that useful for practice._

- Mode switch UI: mode currently follows Major/Minor quality automatically (Ionian/Aeolian) — exposing all 7 diatonic modes as their own live control is still open, deliberately deferred out of `worklog/01-live-key-picker`.
- Tuning switch UI: tuning is fixed at standard for now; switching it live (drop D, 7-string, etc.) reflected on the overlay is still open.
- Notes-per-string subset recipes: a built-in 1-note-per-string overlay on the full Key+Mode view (3 and 2 notes/string landed in `worklog/03-three-notes-per-string-positions/` and `worklog/04-pentatonic-two-notes-per-string-positions/`) — feeds the generalized recipe model below.

## Exercise Subsets

_Carving a specific, useful piece out of a full key/mode overlay — by recipe or by hand — and saving it to come back to._

- Subset recipe model: generalize notes-per-string, fret-range/box, and string-range constraints into a reusable, combinable recipe shape (not one-off implementations).
- Recipe to pattern generation: apply a recipe over a Key+Mode+Tuning+region to a concrete, visualizable, reproducible position set.
- Manual position selection: toggle individual fretboard positions on/off to build an ad hoc Exercise, reviewable before saving, combinable with a recipe-generated starting point.
- Save exercise: name and persist an Exercise (recipe+params or manual), reopenable and re-displayable. First feature that actually needs persistence — see the persistence open question below.
- Open question — degree-subset reuse: pentatonic (major/minor) landed as two hardcoded presets filtering `Mode.degrees` through an unexported helper, not a general mechanism — see `worklog/00-bounded-region-search-and-pentatonic-subsets/`. Worth asking once the subset recipe model above exists: is there a shared shape between degree-level subsetting (pentatonic, dropping scale degrees before projection) and this project's position-level recipes (notes-per-string/fret-range/string-range, filtering after projection)? They look like different layers, but confirm once this project's shape is real rather than assumed.

## Chunk Library

_Turning a saved exercise into a movable, degree-relative pattern ("chunk") not tied to where it was first played, plus a place for that library to live. Base model settled in `worklog/08-chunk-model-design/plan.md` and landed in `worklog/09-current/` — `Slot.t`/`Chunk.t` in `lib/chunk/`._

- ~~Chunk entity model~~ — landed: `Chunk.t = Slot.t list`, `Slot.t = Rest | Note of Degree_reference.t`, no key/mode stored on the Chunk itself (same as `Degree_reference.t`). No separate rhythm representation — a slot's presence/absence *is* the rhythm, every slot equal duration (uneven internal durations deliberately out of scope for now). Distinct from an Exercise (position-level, not degree-relative).
- Exercise to chunk conversion: convert a saved Exercise (or manual sequence) into Slot-based Chunk form.
- Chunk library browser: list saved Chunks, filter/search by key, mode, length, tag.
- Chunk tagging and metadata: name, tags, and a fun/useful rating so favorites surface.
- Chunk reprojection: display the same Chunk at its original position, transposed to a different key, or at a different neck position in the same key — proves the abstraction works both directions. (`Chunk.reframe` covers the degree-relative half already; still needed: running a reframed/transposed Chunk through `Fretboard` to get concrete positions, and a UI to trigger it.)
- Open question — persistence: deliberately deferred until there's a chunk model worth using day to day — see the Cross-Cutting persistence question below.

## Chord Progressions

_A chord progression as its own movable, degree-relative object — not just a Song's attribute — so the same shape (e.g. "I-IV-V") is reusable across keys and songs. A lot of music has several progressions within one song anyway, and a progression maps cleanly onto a song but not the reverse. Base model landed in `worklog/09-current/` — `Chord_progression.t` in `lib/chunk/chord_progression.ml`._

- ~~Chord progression entity model~~ — landed: `Chord_progression.t = int list` of scale-degree roots (e.g. `[1; 4; 5]`), quality inferred diatonically from the underlying Key+Mode for now — no explicit per-step quality override (borrowed chords, secondary dominants) yet, deliberately deferred.
- ~~Chunk-over-progression reprojection~~ — landed: `Chord_progression.apply_to_chunk` repeats a Chunk once per progression step via `Chunk.reframe`, reinterpreting its own degree numbers relative to that step's root (the same rotate-by-N-degrees move `Mode.degrees` already makes when picking which note counts as a mode's own root, generalized to an arbitrary scale-degree anchor, wrapping mod 7 with an octave carry).
- Open question — saved or ephemeral: is a Chunk-applied-to-a-Progression worth persisting as its own nameable thing, or always computed on the fly from the two separately-saved pieces? Deliberately unresolved alongside the general persistence question below.

## Chunk-to-Fretboard Solvers

_Turning an abstract, harmonically-rooted Chunk into one or more concrete fingerings — the "many ways to play this" layer between the abstract Chunk and something practiceable. Base solver landed in `worklog/09-chunk-library-core` — `Chunk_solver` in `lib/chunk/chunk_solver.ml`._

- ~~Solver strategies~~ — landed a first one: `Chunk_solver.positions` takes a starting anchor, a hard per-move fret cap, and a sequence of already-harmonically-resolved `Degree_reference.t` (from `Chunk.reframe`), and returns *every* valid way to realize them as concrete positions — ranked by a raw `|fret delta| + |string delta|` distance metric between consecutive notes, no same-string bonus (checked and dropped after a candidate example turned out not to need one). This resolved `docs/chunk-and-fretboard-model.md` §8's "how does a multi-note chunk anchor itself" question for the case of solving *one chord's own instance* of a Chunk: each chord's own triad is solved fresh from the same starting anchor (not chained from the previous chord's ending note).
- ~~Confirmed — shared-anchor projection doesn't track melodic direction~~ and ~~chaining across a Chord_progression compounds with `Chunk.reframe`'s octave carry~~ — resolved, but not the way first attempted. The first `Chunk_solver` fix ignored each `Degree_reference`'s own `octave` field entirely, reasoning that its own bounded search already determined register — that broke every non-tonic chord in a real I-vi-IV-V run (e.g. vi's reframed-and-wrapped notes landed a full octave above where they should, since the search just found whatever was hand-closest rather than the pitch the wrap actually meant). The real fix: resolve every note's own true, ever-increasing semitone height from its degree+octave *before* choosing any position (`Fretboard.target_pitch_class`'s pitch class + 12 semitones per octave the degree carries) — reframing shifts every note in a chunk by the same amount, so this exactly preserves whatever interval the chunk was authored with, wrapped octave or not. Only *which string* reaches that already-determined pitch is chosen positionally. Confirmed with a full I-vi-IV-V run: every chord now resolves to a correct, comfortable triad (right note names, right intervals, ascending) with no cross-chord drift. Two regression tests (`test/chunk/chunk_solver_test.ml`) lock this in: a reframed wrapped-octave case resolves to the right minor-third-then-major-third, and a deliberately authored octave leap resolves to exactly 12 semitones.
- Modal-position-constrained strategy: a second strategy worth adding alongside `Chunk_solver`'s free/bounded search — constrain candidates to one of the 7 diatonic (or 5 pentatonic / 3 arpeggio) notes-per-string positions already computed by `Fretboard`, for players who want to stay within a known box/shape rather than whatever the distance metric finds. Surfaced from a real example: the shared-anchor (pre-`Chunk_solver`) projection of an 8-note arpeggio chunk solved to a fingering that also happened to fit cleanly inside the existing Locrian 3-notes-per-string position — confirmed multiple equally-valid fingerings of the same Chunk is the normal case, not an edge case.
- Batch browsing: `Chunk_solver.positions` already returns every valid shape sorted by total distance, not just the best one — "resolve to multiple shapes and pick one from the batch to practice" is the stated goal, but nothing yet lets a player actually browse that batch; `bin/main.ml`'s demo just takes the head of the list today.
- Pick-and-practice flow: choose one solved candidate (from the batch above) to view as tab + fretboard shape and practice, using the existing tab/fretboard viewer — this is where a solved Chunk starts looking like today's Exercise concept.

## Permutation Engine

_Taking chunks and doing something with them beyond storage — altering notes/rhythm, sequencing several together, swapping pieces, polyrhythm and phrasing across the barline._

- Note-level variation rules: add/remove a note (e.g. truncating a chunk's trailing notes), substitute a degree, each producing a new derived Chunk.
- Rhythmic re-permutation: map the same Slot sequence over a different number of underlying time subdivisions (e.g. a 4-slot chunk over a 3-grouping = a 4-over-3 polyrhythm) while preserving degree content — confirmed this is the whole "rhythm" story for a Chunk, no separate rhythm representation needed.
- Variation lineage tracking: a derived Chunk records its parent and the rule (+ params) that produced it — confirmed strict lineage (a tree, not a DAG) for now.
- ABC arrangement builder: label Chunks (A, B, C...), sequence them into an Arrangement, preview as a whole.
- Arrangement slot swapping: swap a labeled slot for an alternate Chunk/variation without rebuilding the rest, try multiple combinations. ("Slot" here means an arrangement position, unrelated to a Chunk's own `Slot.t` above — worth keeping straight.)
- Polyrhythm and barline-crossing arrangements: support phrases that don't align to measure boundaries and at least one polyrhythmic layering (e.g. 3-over-4), still coherently previewable.

## Tab-to-Chunk Extraction

_Closing the loop between a real song and the chunk system — pulling a phrase out of a tab and turning it into a reusable chunk._

- Tab region to chunk extraction: convert a selected Tab region into an abstract Chunk given a declared Key/Mode, correctly carrying rhythm (not just pitches).
- Extraction round-trip verification: an extracted Chunk re-projected back to its source position matches it; re-projected elsewhere is musically coherent; extraction and projection share code (no parallel implementations to drift apart).

## Songs & Practice Context

_Giving a song its own identity (key, tempo, chords) so exercises and chunks can be viewed against it, not just in the abstract._

- Song entity model: title, Key, Mode, BPM, time signature, one or more Chord Progressions (see above — a Song references Chord Progressions, not the reverse; most songs use more than one), links to one or more Tabs; create/save/reopen.
- Manual song metadata entry: manual UI for Key/BPM/chord progression(s) (automatic extraction is the open question below).
- Bind exercise to song context: view an Exercise/Chunk/Arrangement transposed into a Song's Key, target BPM shown alongside (visually, not audibly) — binding doesn't mutate the underlying Exercise/Chunk.
- Practice-in-context view: show a Chunk/Exercise/Arrangement against the Song's chord progression(s) per measure/section, across a full song length, updating if the bound Chunk/Exercise changes.
- Open question — automatic song metadata extraction: manual-only vs. extracted-from-tab-metadata vs. hybrid (extraction proposes, user confirms). Unresolved; manual entry above ships first regardless.

## Cross-Cutting

_Applies to everything rather than belonging to one grouping._

- Visual polish pass: recurring, not one-shot — revisit styling/layout/consistency (typography, color, spacing across fretboard/tab/library views) after each active project, since "pretty" is a stated quality goal.
- Performance/startup checks: recurring — check startup time and browsing responsiveness after any project that adds persisted data, since fast startup is a stated quality goal.
- Open question — persistence layer: file-per-entity (human-readable, git-friendly, slow relationship queries) vs. a single structured store (enforced integrity, fast queries, not hand-editable) vs. hybrid (files for authoring + a derived index for queries). Deliberately unresolved until "save exercise" forces it.
