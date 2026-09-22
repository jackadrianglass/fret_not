open! Base

type t =
  | Rest
  | Note of Degree_reference.t

let equal a b =
  match (a, b) with
  | Rest, Rest -> true
  | Note a, Note b -> Degree_reference.equal a b
  | Rest, Note _ | Note _, Rest -> false
;;
