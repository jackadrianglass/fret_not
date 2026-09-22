# Plan

Not an implementation pass — this entry is a design conversation about the
`Chunk` model, kept as its own worklog slice because it's substantive
enough to want a record, same as any other. The goal: get a base model and
a solid `todo.md` update out of a real conversation about what's actually
wanted, rather than guessing ahead of that.

# Context — where the model and the app actually stand today

**Built:** the abstract↔concrete bridge
(`docs/chunk-and-fretboard-model.md` §1-6) — `Key`/`Mode`/`Scale_degree`
(abstract, no fretboard knowledge), `Tuning`/`Fretboard_position`
(concrete, no scale knowledge), and `Fretboard.to_positions`/`of_position`
as the only things touching both, anchored by a single `anchor_position`.
Three concrete projections sit on top of that bridge: 3-notes-per-string
(7 positions, one per mode), 2-notes-per-string pentatonic (5 positions),
1-note-per-string arpeggio (3 positions, root/3rd/5th). A bare-bones tab
viewer and fretboard viewer render whichever of those is currently
selected, full-screen, no rhythm.

**Not built at all yet:**
- `Chunk` itself. `Degree_reference.t` (`{ degree; octave; alteration }`)
  is the atomic unit `todo.md`'s Chunk Library section says a Chunk will
  be a sequence *of*, alongside "Rhythm Cells" — but `Rhythm_cell` doesn't
  exist as a type anywhere; it's a name in a todo bullet, nothing else.
- Any notion of a **chord** as its own thing. The model has scale/mode
  harmony (`Key`, `Mode`) and one fixed chord shape derived from it
  (`Arpeggio`, root/3rd/5th of the current key) — nothing like "Dm7" or
  "G7#9" as an independent quality nameable apart from a key.
- Persistence, arrangement, song/progression binding — all still
  `todo.md` bullets under Chunk Library / Permutation Engine / Songs &
  Practice Context, untouched.

**Already-flagged open threads that bear directly on this conversation**
(`docs/chunk-and-fretboard-model.md` §7):
- *How a multi-note chunk anchors itself* — `to_positions` takes one
  anchor per call; does a whole chunk anchor to one shared position, or
  does each note chain off wherever the previous note landed? The doc
  explicitly punts this to "04-chunk-library's call to make with real
  chunk examples in hand" — which is now.
- *Chromatic reverse spelling* — `of_position` raises on a non-diatonic
  pitch rather than guessing a spelling. Relevant if a chunk needs to
  represent a chromatic passing tone.
- Degree-level subsetting (pentatonic, arpeggio — filtering
  `Scale_degree.t` before projection) is already established as a
  *different layer* from position-level recipes (notes-per-string,
  filtering `Fretboard_position.t` after projection) — confirmed in
  `worklog/00-bounded-region-search-and-pentatonic-subsets/`. Worth
  keeping in mind as a precedent for where a new "chord" concept would
  slot in: probably another abstract-side layer, not a fretboard-side one.

# Questions

## Use cases

Give me 2-3 concrete chunks actually in mind (a lick, a comping pattern, a
scale run, whatever) — different enough from each other to stress-test the
model differently. What do they look like as notes+rhythm, roughly?

Is the primary use case practice material to drill (extending what the
fretboard/tab viewer already do), building blocks for arranging/composing
(`todo.md`'s Permutation Engine: ABC arrangement builder, slot swapping),
or genuinely both from the start?

Is this just for personal use, or do chunks want to be shareable/
exportable at some point? (Shapes how much the model needs to commit to
now vs. defer — `todo.md` already flags import/export as deliberately
deferred.)

### Answer

Yes they can be licks, patterns, scale runs, rhythmic ideas, chord sequences

> Use case 1

I'm currently practicing major and minor arpeggio shapes. One melody that I'm trying out is
basically a 8 note sequence on the arpeggio starting at the top note (this is in root position)
top -> 5th -> 3rd -> root -> 3rd -> 5th -> top -> 5th

I can apply this to a chord progression where I repeat the chunk in a 1, 4, 5 pattern. A variant
of this chunk could be taking out the last two notes
top -> 5th -> 3rd -> root -> 3rd -> 5th

> Use case 2

A rhythmic chunking idea could be doing double note single in groupings. if we're in 16th note triples and O is played and - is rest, then it looks like
OOO-O-

You can do similar things with groupings of 5s

OOO-OOO-O

> Use case 3

I'm looking through a particular song and I find a lick that I really like. I want to be able to pull that lick out, understand it easily, and integrate the idea into my vocabulary

The idea behind the tool is to author practice drills which can easily be integrated into your existing vocabulary in a way that you can improvise and compose with. This isn't meant to be a composition tool, but rather a test bed to try out ideas that could lead to a composition.

## Rhythmic subdivision

Should a Chunk own exactly one rhythm (with "reinterpret across a
different subdivision" — `todo.md`'s Rhythmic re-permutation — producing a
*new*, derived Chunk), or should rhythm be a separate, swappable layer
from the start (the same degree sequence paired with different rhythm
layers interchangeably, no new Chunk needed to swap one)?

Grid-locked to a time signature/measure, or free-time-capable (durations
as ratios, not measure-anchored)? `todo.md`'s own "future import/export
constraints" question already flags "rhythm not grid-locked" as a
property worth protecting now.

Do rests need to be first-class (a chunk can have silence in it), or is a
chunk always a dense run of notes?

### Answer

I'm thinking for now, let's do sequences of notes (including rests) which can be applied over subdivisions. So if I have a 4 note chunk and I apply it over triples, I get a 4 over 3 polyrhythm

## Base chord / position

This seems to bundle two different anchors — worth pulling apart:

**Physical anchor** (§7's open question, directly): for one of the
example chunks, if it moved to a different string/fret, would every note
shift by the same amount (one shared anchor), or would it "walk" — each
note's position depending on where the previous one landed (chained
anchor)? Real 3nps/2nps/1nps positions today are built by chaining, for
what it's worth, but that's a fretboard-geometry walk, not a chunk being
projected.

**Harmonic anchor**: is "base chord" the scale/mode a chunk is already
relative to (nothing new needed), or is an actual `Chord` concept wanted —
quality (maj7, m7, dom7...) independent of a key, closer to how `Arpeggio`
builds a fixed triad but generalized and nameable on its own? If the
latter, that's a real new piece of the abstract side, not configuration on
the existing one.

### Answer

I think that we're going to have to play with these ideas more before this can be answered. My initial idea was to have the chunk routed in the harmonic anchor and have some solvers to offer ways of applying that to the fret board. That way, I can learn options of how to navigate the fretboard

## Chord progressions

Is the near-term interest "a chunk that knows how to reproject itself as
the chord changes under it within one song" (matches `todo.md`'s Songs &
Practice Context: chunk bound to a Song's chord sequence), or "a chunk *is*
defined relative to a progression" (the chunk's own identity includes a
sequence of chord contexts, not one key/mode)? Those are pretty different
commitments.

Does a progression live only at the Song level (where `todo.md` has it
today), or does it want to be its own movable object — "a ii-V-I in any
key," mirroring how Chunks themselves are movable?

### Answer

I'm thinking it's own movable object. That would map easily to the song but vice versa isn't true. And a lot of music has many chord progressions within a single song anyways

## Variants

`todo.md`'s Permutation Engine already sketches note-level variation rules
(add/remove a note, substitute a degree) and lineage tracking (a derived
Chunk records its parent + the rule that produced it). Does that match
what "variants" means here, or is there a different axis in mind — e.g.
different *fingerings* of the same abstract note sequence, rather than
different notes?

Strict lineage (one parent per variant, a tree), or could a variant
reasonably combine ideas from more than one chunk (a DAG)?

### Answer

Let's just make a strict lineage to start with. I feel like we could easily confuse ourselves with a DAG

# Follow-up questions (after first round of answers)

## Rhythm as its own layer

Your rhythmic-subdivision answer (reusing a 4-note chunk over a
triplet/3-grouping to get a 4-over-3 polyrhythm) and the pure-rhythm
example in use case 2 ("OOO-O-", no pitches at all) both point the same
way: pitch/degree content and rhythm/slot content sound like two
independent, separately-swappable things, not one fused Chunk that owns
a single rhythm. Is that right? And if so — is a Chunk the *combination*
(a specific degree-sequence + a specific rhythm pattern, already
paired), with the rhythm pattern itself also being its own named,
reusable thing (its own little library, mirroring the Chunk Library), or
is "Chunk" reserved for the degree-sequence side only, with rhythm
applied to it more like a lens/transform at display time?

Are all the "slots" in a rhythm pattern always equal duration (a uniform
grid — matches both your examples), or does a chunk ever need *uneven*
internal durations (e.g. a dotted note followed by two short ones,
within the same chunk)? Scoping to "uniform slots, note count is the
only rhythmic parameter" for v1 seems to match what you described —
confirming before I run with it.

### Answer

I don't think that we need a seperate rhythmic layer. You can easily model the "rhythmic idea" with just an arbitrary scale degree. In that example, we could do the first degree

111-1-

and then map it over a chord progression 1, 4, 5 in a 3/4 16 note triplet bar

111-1- 444-4- 555-5-

so the slots are always equal duration

## The "solver" idea

You said the chunk should be rooted in the harmonic anchor with "some
solvers to offer ways of applying that to the fretboard" so you can
learn options for navigating the neck. That's a nice way to sidestep the
shared-anchor-vs-chained-anchor fork from before — instead of picking
one, a solver could offer *both* as different strategies (plus others,
like "stay within these 4 frets" or "stay on these 3 strings") and hand
you several candidate fingerings to choose from, closer to how
`to_positions` already returns multiple candidate strings sorted by
distance, generalized to a whole multi-note sequence. Is that the shape
you're picturing, or something more specific (e.g. a particular
constraint-solving approach you already have in mind)?

### Answer

Yeah exactly. Then within a given session, I could pick one of the variants to practice, see the tabs for it and the shapes, then practice that

## Chord progressions as scale-degree-relative

In use case 1, "repeat the chunk in a 1, 4, 5 pattern" reads like the
progression itself is scale-degree-relative — I, IV, V of whatever key
it's in — the same move `Degree_reference` already makes for single
notes, generalized to a sequence of chord roots. Does each step in a
progression need its own explicit chord quality (e.g. distinguishing a
borrowed iv from a diatonic IV, or a secondary dominant), or is quality
always inferred from the key/mode automatically for now (plain
roman-numeral degrees, no per-step override)?

### Answer

Let's infer them for now

# Second follow-up round

Your rhythm answer folds "rhythm" entirely into ordinary degree content
(a rest-or-degree slot sequence, all equal duration, no separate Rhythm
type) — that's a real simplification worth carrying forward: `todo.md`'s
"Rhythm Cell" idea goes away, replaced by `Slot = Rest | Note of
Degree_reference.t` and a chunk being a `Slot.t list`. "Applying over a
subdivision" becomes purely "how many real time-grid units does this
slot list span," not a separate object.

Your progression example (`111-1- 444-4- 555-5-`) also clarified how a
chunk meets a progression: the same 6-slot pattern repeats once per
progression step, with its own degree numbers reinterpreted relative to
that step's chord root — degree "1" against the IV step means "the 4th
scale degree of the underlying key," the same move `Mode.degrees`
already makes when reframing which note counts as a mode's own root.

## Two narrow confirms

Is that reframing-per-step reading right — each progression step is like
a fresh local anchor the chunk's degrees get reinterpreted against, the
same shape as a mode picking a different starting degree?

When a chunk gets applied to a progression, is the result something you
want to save as its own artifact (nameable, reopenable — "chunk X over
progression Y"), or is that always an ephemeral, in-the-moment
combination, with the chunk and the progression remaining the only two
things actually saved?

### Answer

Reframing-per-step reading confirmed (no objection raised). Persistence
of a chunk-applied-to-a-progression: deferred until there's a chunk
model worth using day to day — same deferral as the model's persistence
question generally (see `todo.md` Cross-Cutting).

# Model sketch

Not implemented yet — this is the sketch the two Q&A rounds above settled
on, written down so `lib/` has something concrete to build toward. See
`todo.md` for the resulting backlog items.

## Slot and Chunk

```
Slot.t = Rest | Note of Degree_reference.t
Chunk.t = Slot.t list
```

No separate rhythm type. A slot's presence/absence *is* the rhythm; every
slot is equal duration (confirmed — uneven internal durations explicitly
out of scope for now). "Apply a Chunk over a subdivision" (the 4-over-3
polyrhythm idea) means picking how many real time-grid units the whole
slot list spans — a number, not a new object.

Naming note: `todo.md`'s existing "Arrangement slot swapping" bullet uses
"slot" colloquially for an arrangement position, unrelated to this new
`Slot.t`. Worth keeping straight — this is a *Chunk* slot.

## Chord_progression

```
Chord_progression.t = int list   (* scale-degree roots, e.g. [1; 4; 5] *)
```

Quality inferred diatonically from the underlying Key+Mode (confirmed —
no explicit per-step quality/borrowed-chord/secondary-dominant support
yet). Reprojecting a Chunk over one progression step reuses the same
rotate-by-N-degrees move `Mode.degrees` already makes when picking which
note counts as a mode's own root — the step's scale degree becomes a
fresh local anchor each of the Chunk's own degree numbers gets
reinterpreted against, wrapping mod 7.

## Chunk-to-fretboard solvers

Deliberately not typed yet ("we're going to have to play with these
ideas more" — still true). The settled shape: a solver takes a Chunk
(harmonically rooted, no fretboard knowledge) and an anchor, and offers
*multiple* candidate whole-chunk fingerings rather than committing to
one — generalizing `Fretboard.to_positions`'s single-note, single-anchor,
multiple-candidate-strings shape up to a whole multi-slot sequence. At
least two strategies: shared-anchor (every note anchored to the same
position) and chained/walking (each note anchors off wherever the
previous one landed) — this is `docs/chunk-and-fretboard-model.md` §7's
still-open anchoring question, reframed as "offer both as solver
strategies" rather than picking one. Picking a solved candidate to view
as tab + fretboard shape and practice is where this starts looking like
today's Exercise concept.

# References

None.
