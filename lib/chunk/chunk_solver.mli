open! Base

val distance : Fretboard_position.t -> Fretboard_position.t -> int
(** |fret delta| + |string delta| - a proxy for how far a hand has to move
    between two positions. Deliberately just this and nothing fancier (no
    same-string bonus, no real fret-spacing physics) - see
    worklog/09-chunk-library-core for why. *)

val positions :
     key:Key.t
  -> mode:Mode.t
  -> tuning:Tuning.t
  -> start_anchor:Fretboard_position.t
  -> max_fret_distance:int
  -> Degree_reference.t list
  -> Fretboard_position.t list list
(** Every way to realize this sequence of Degree References as concrete,
    hand-comfortable fretboard positions - one "shape" per element of the
    returned list, sorted by total distance (nearest-overall first, see
    distance).

    The first note resolves to the nearest occurrences of its own pitch class
    within max_fret_distance frets of start_anchor. Every later note's absolute
    pitch (including its own octave, relative to the first note's) is fully
    determined by the sequence's own harmonic content - each note's
    degree+octave is resolved to a real, ever-increasing semitone height (via
    Fretboard.target_pitch_class combined with the degree's own octave) before
    any fretboard position is chosen, so the melodic shape a Chunk was authored
    with (an interval, an intentional octave leap, ...) is preserved exactly,
    however that specific pitch is voiced. Only *which string* reaches that
    already-fully-determined pitch is chosen positionally, constrained to within
    max_fret_distance of the previous note.

    Returns [] if no combination stays within max_fret_distance throughout. *)
