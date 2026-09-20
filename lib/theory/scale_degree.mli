open! Base

type t =
  { degree : int
  ; pitch_class : Pitch_class.t
  }

val equal : t -> t -> bool
val compare : t -> t -> int
