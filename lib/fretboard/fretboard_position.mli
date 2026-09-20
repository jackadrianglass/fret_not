open! Base

type t =
  { string_index : int
  ; fret : int
  }

val equal : t -> t -> bool
val compare : t -> t -> int
