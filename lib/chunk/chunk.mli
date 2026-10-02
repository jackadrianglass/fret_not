open! Base

type t = Slot.t list

val equal : t -> t -> bool

val reframe : t -> root_degree:int -> t
