open! Base

type t =
  | Natural
  | Sharp
  | Flat
  | Double_sharp
  | Double_flat
[@@deriving eq]

val semitones : t -> int
val of_semitones : int -> t option
val to_string : t -> string

val compare : t -> t -> int
(** By semitones, not declaration order. *)
