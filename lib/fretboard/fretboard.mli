open! Base

val root_note : key:Key.t -> Note.t
(** The tonic as a note in octave register 0; the octave anchor all degree
    resolution on this module is relative to. *)

val degree_note : key:Key.t -> mode:Mode.t -> Degree_reference.t -> Note.t

val positions_of_semitone :
  instrument:Instrument.t -> target_semitone:int -> Fretboard_position.t list
(** Every playable position sounding exactly that semitone, one per string at
    most, ascending by string. *)

val degree_positions :
     instrument:Instrument.t
  -> key:Key.t
  -> mode:Mode.t
  -> Degree_reference.t
  -> Fretboard_position.t list
(** Every playable position sounding that exact degree and octave, ascending by
    string. *)

val degree_at :
     instrument:Instrument.t
  -> key:Key.t
  -> mode:Mode.t
  -> Fretboard_position.t
  -> Degree_reference.t option
(** None when the position sounds off-key. *)

val degrees_in_window :
     instrument:Instrument.t
  -> key:Key.t
  -> mode:Mode.t
  -> min_fret:int
  -> max_fret:int
  -> (Fretboard_position.t * Degree_reference.t) list
(** Every playable in-key position with its degree, ascending by string then
    fret. *)
