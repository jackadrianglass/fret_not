open! Base

val distance : Fretboard_position.t -> Fretboard_position.t -> int

val positions :
     key:Key.t
  -> mode:Mode.t
  -> tuning:Tuning.t
  -> start_anchor:Fretboard_position.t
  -> max_fret_distance:int
  -> Degree_reference.t list
  -> Fretboard_position.t list list
