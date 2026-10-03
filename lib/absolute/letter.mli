open! Base

type t =
  | C
  | D
  | E
  | F
  | G
  | A
  | B
[@@deriving eq, ord]

val all : t list
val index : t -> int
val of_index : int -> t
val offset : t -> int -> t
val to_pitch_class : t -> Pitch_class.t
val to_string : t -> string
