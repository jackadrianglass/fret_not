open! Base

type t =
  | Natural
  | Sharp
  | Flat
  | Double_sharp
  | Double_flat

val semitones : t -> int
val to_string : t -> string
val equal : t -> t -> bool
val compare : t -> t -> int
