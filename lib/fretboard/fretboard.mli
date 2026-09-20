open! Base

val to_positions :
     key:Key.t
  -> mode:Mode.t
  -> tuning:Tuning.t
  -> anchor_position:Fretboard_position.t
  -> Degree_reference.t
  -> Fretboard_position.t list
(** anchor_position defines what "octave 0" means for this call: the nearest
    occurrence of the target pitch class to that position, ties broken upward.
    The same Degree_reference resolves to different Fretboard Positions
    depending on the anchor — that's what lets an abstract pattern be projected
    anywhere on the neck without editing its own data. Returns one position per
    string that can reach the target (each string reaches a given absolute pitch
    at exactly one fret, or not at all), sorted by fret-distance from
    anchor_position. *)

val positions_in_window :
     key:Key.t
  -> mode:Mode.t
  -> tuning:Tuning.t
  -> anchor_position:Fretboard_position.t
  -> min_fret:int
  -> max_fret:int
  -> Fretboard_position.t list
(** Every position of every degree in the mode that falls within
    [min_fret, max_fret], anchored the same way as to_positions. The octave
    range searched per degree is computed from the window and tuning, not
    guessed - unlike calling to_positions once per (degree, octave) and
    filtering by hand, this can't silently miss positions on a wide neck or an
    unusual tuning. Sorted by (string_index, fret). *)

val of_position :
     key:Key.t
  -> mode:Mode.t
  -> tuning:Tuning.t
  -> anchor_position:Fretboard_position.t
  -> Fretboard_position.t
  -> Degree_reference.t
(** Inverse of to_positions, anchored the same way — round-tripping (degree ->
    position -> degree) via the same anchor_position returns the original
    degree, for the diatonic case (alteration = 0). Raises if the position's
    pitch class isn't one of the mode's 7 diatonic degrees; chromatic positions
    aren't representable as a Degree_reference yet. *)
