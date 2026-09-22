open! Base

type t =
  | Rest
  | Note of Degree_reference.t

val equal : t -> t -> bool
