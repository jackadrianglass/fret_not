open! Base

type t

val create : Pitch_class.t list -> t

val string_count : t -> int
val pitch_class_at : t -> string_index:int -> fret:int -> Pitch_class.t

val relative_semitone : t -> string_index:int -> fret:int -> int

val standard : t
val drop_d : t
val standard_seven_string : t
