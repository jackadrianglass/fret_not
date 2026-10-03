open! Base

type t =
  | Rest
  | Note of Degree_reference.t
[@@deriving eq]
