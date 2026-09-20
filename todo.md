# Todo

Ideas discussed but not started. Grouped loosely by theme for scanability — no priority or dependency implied by order or grouping. When something here becomes the active thread, it graduates to a `worklog/` entry and gets dropped from here once it actually lands.

## Tab Editor Core

_Viewing and editing a tab with real rhythmic notation, not just fret numbers — stops short of chunk extraction or import/export._

- Tab event model: a Fretboard Position + Rhythm Cell + optional articulation (slide, bend, hammer-on), grouped into measures under a time signature. Format-agnostic — no import/export assumptions baked in.
- Tab entry/editing UI: place, move, delete, and re-duration notes directly on strings/frets; add/remove measures.
- Rhythm notation rendering: stems/beams/durations rendered alongside the tab, staying aligned with it; rests representable.
- Tab cursor/scrubbing: a visual-only cursor stepping through Tab Events — no audio.
- Tab region selection: select a contiguous measure range, highlighted, readable back as an ordered list of Tab Events. Feeds tab-to-chunk extraction later.
- Open question — future import/export constraints: Ultimate Guitar text/Guitar Pro/MusicXML import/export is deliberately deferred; write down the constraints the Tab Event model must satisfy now (e.g. articulation optional, rhythm not grid-locked) so adding an importer later doesn't force a rework. No importer gets built as part of this.

## Fretboard Viewer

_Seeing a key or mode laid out across the whole neck, with the controls that make that useful for practice._

- Mode switch UI: mode currently follows Major/Minor quality automatically (Ionian/Aeolian) — exposing all 7 diatonic modes as their own live control is still open, deliberately deferred out of `worklog/01-live-key-picker`.
- Tuning switch UI: tuning is fixed at standard for now; switching it live (drop D, 7-string, etc.) reflected on the overlay is still open.
- Notes-per-string subset recipes: built-in 3/2/1-notes-per-string overlays on the full Key+Mode view — feeds the generalized recipe model below.

## Exercise Subsets

_Carving a specific, useful piece out of a full key/mode overlay — by recipe or by hand — and saving it to come back to._

- Subset recipe model: generalize notes-per-string, fret-range/box, and string-range constraints into a reusable, combinable recipe shape (not one-off implementations).
- Recipe to pattern generation: apply a recipe over a Key+Mode+Tuning+region to a concrete, visualizable, reproducible position set.
- Manual position selection: toggle individual fretboard positions on/off to build an ad hoc Exercise, reviewable before saving, combinable with a recipe-generated starting point.
- Save exercise: name and persist an Exercise (recipe+params or manual), reopenable and re-displayable. First feature that actually needs persistence — see the persistence open question below.
- Open question — degree-subset reuse: pentatonic (major/minor) landed as two hardcoded presets filtering `Mode.degrees` through an unexported helper, not a general mechanism — see `worklog/00-bounded-region-search-and-pentatonic-subsets/`. Worth asking once the subset recipe model above exists: is there a shared shape between degree-level subsetting (pentatonic, dropping scale degrees before projection) and this project's position-level recipes (notes-per-string/fret-range/string-range, filtering after projection)? They look like different layers, but confirm once this project's shape is real rather than assumed.

## Chunk Library

_Turning a saved exercise into a movable, degree-relative pattern ("chunk") not tied to where it was first played, plus a place for that library to live._

- Chunk entity model: a Chunk is an ordered sequence of Degree References + Rhythm Cells, scoped to Key+Mode+Time Signature — degree-relative, not fixed fret positions, distinct from an Exercise.
- Exercise to chunk conversion: convert a saved Exercise (or manual sequence) into degree-relative Chunk form, assigning rhythm during conversion.
- Chunk library browser: list saved Chunks, filter/search by key, mode, length, tag.
- Chunk tagging and metadata: name, tags, and a fun/useful rating so favorites surface.
- Chunk reprojection: display the same Chunk at its original position, transposed to a different key, or at a different neck position in the same key — proves the abstraction works both directions.

## Permutation Engine

_Taking chunks and doing something with them beyond storage — altering notes/rhythm, sequencing several together, swapping pieces, polyrhythm and phrasing across the barline._

- Note-level variation rules: add/remove a note, substitute a degree, each producing a new derived Chunk.
- Rhythmic re-permutation: reinterpret a Chunk's notes across a different rhythmic grouping (e.g. triplets vs. sixteenths) while preserving degree content.
- Variation lineage tracking: a derived Chunk records its parent and the rule (+ params) that produced it, traceable across generations.
- ABC arrangement builder: label Chunks (A, B, C...), sequence them into an Arrangement, preview as a whole.
- Arrangement slot swapping: swap a labeled slot for an alternate Chunk/variation without rebuilding the rest, try multiple combinations.
- Polyrhythm and barline-crossing arrangements: support phrases that don't align to measure boundaries and at least one polyrhythmic layering (e.g. 3-over-4), still coherently previewable.

## Tab-to-Chunk Extraction

_Closing the loop between a real song and the chunk system — pulling a phrase out of a tab and turning it into a reusable chunk._

- Tab region to chunk extraction: convert a selected Tab region into an abstract Chunk given a declared Key/Mode, correctly carrying rhythm (not just pitches).
- Extraction round-trip verification: an extracted Chunk re-projected back to its source position matches it; re-projected elsewhere is musically coherent; extraction and projection share code (no parallel implementations to drift apart).

## Songs & Practice Context

_Giving a song its own identity (key, tempo, chords) so exercises and chunks can be viewed against it, not just in the abstract._

- Song entity model: title, Key, Mode, BPM, time signature, chord sequence, links to one or more Tabs; create/save/reopen.
- Manual song metadata entry: manual UI for Key/BPM/chord sequence (automatic extraction is the open question below).
- Bind exercise to song context: view an Exercise/Chunk/Arrangement transposed into a Song's Key, target BPM shown alongside (visually, not audibly) — binding doesn't mutate the underlying Exercise/Chunk.
- Practice-in-context view: show a Chunk/Exercise/Arrangement against the Song's chord sequence per measure/section, across a full song length, updating if the bound Chunk/Exercise changes.
- Open question — automatic song metadata extraction: manual-only vs. extracted-from-tab-metadata vs. hybrid (extraction proposes, user confirms). Unresolved; manual entry above ships first regardless.

## Cross-Cutting

_Applies to everything rather than belonging to one grouping._

- Visual polish pass: recurring, not one-shot — revisit styling/layout/consistency (typography, color, spacing across fretboard/tab/library views) after each active project, since "pretty" is a stated quality goal.
- Performance/startup checks: recurring — check startup time and browsing responsiveness after any project that adds persisted data, since fast startup is a stated quality goal.
- Open question — persistence layer: file-per-entity (human-readable, git-friendly, slow relationship queries) vs. a single structured store (enforced integrity, fast queries, not hand-editable) vs. hybrid (files for authoring + a derived index for queries). Deliberately unresolved until "save exercise" forces it.
