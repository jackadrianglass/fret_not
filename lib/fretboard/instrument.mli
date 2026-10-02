open! Base

type t

(** A physical instrument: a tuning plus the frets actually available on each
    string. Everything that asks "can this be played here" goes through an
    instrument. *)

val create : tuning:Tuning.t -> fret_count:int list -> t
val create_uniform : tuning:Tuning.t -> fret_count:int -> t
val tuning : t -> Tuning.t
val string_count : t -> int
val fret_count : t -> string_index:int -> int
val max_fret : t -> int
val playable : t -> Fretboard_position.t -> bool
val semitone_at : t -> Fretboard_position.t -> int
val pitch_class_at : t -> Fretboard_position.t -> Pitch_class.t

val range : t -> int * int
(** Lowest and highest sounding semitone reachable on the instrument. *)
