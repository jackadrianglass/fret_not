open! Base

type t

val create : Pitch_class.t list -> t
(** Open-string pitch classes, ordered low to high (string_index 0 is the
    lowest-pitched string). *)

val string_count : t -> int
val pitch_class_at : t -> string_index:int -> fret:int -> Pitch_class.t

val relative_semitone : t -> string_index:int -> fret:int -> int
(** True semitone height above string_index 0's open pitch (not modulo 12) —
    lets callers compare real pitch distance between two positions on different
    strings, which pitch_class_at alone can't do. Assumes each string's open
    pitch is a small ascending interval above the previous one, true of every
    real guitar tuning. *)

val standard : t
val drop_d : t
val standard_seven_string : t
