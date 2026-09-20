open! Base

type t =
  { degree : int
  ; octave : int
  ; alteration : int
  }

val equal : t -> t -> bool
val compare : t -> t -> int
