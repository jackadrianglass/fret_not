open! Base

type t

(** Open-string pitches from lowest to highest, as absolute SPN notes. *)

val create : Note.t list -> t
val string_count : t -> int
val open_note : t -> string_index:int -> Note.t
val open_semitone : t -> string_index:int -> int

val semitone_at : t -> string_index:int -> fret:int -> int
(** Sounding semitones above C0 of a position: the open string's semitone plus
    the fret. *)

val pitch_class_at : t -> string_index:int -> fret:int -> Pitch_class.t

val retune_string : t -> string_index:int -> semitones:int -> t
(** Shift one string's open note by semitones, spelling the result with the
    smallest accidental available. Drop tunings derive from [standard]. *)

val standard : t
val drop_d : t
val standard_seven_string : t
val bass_four : t
