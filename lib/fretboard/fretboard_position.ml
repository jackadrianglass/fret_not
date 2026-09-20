open! Base

type t =
  { string_index : int
  ; fret : int
  }

let equal a b =
  Int.equal a.string_index b.string_index && Int.equal a.fret b.fret
;;

let compare a b =
  match Int.compare a.string_index b.string_index with
  | 0 -> Int.compare a.fret b.fret
  | nonzero -> nonzero
;;
