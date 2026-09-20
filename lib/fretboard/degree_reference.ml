open! Base

type t =
  { degree : int
  ; octave : int
  ; alteration : int
  }

let equal a b =
  Int.equal a.degree b.degree
  && Int.equal a.octave b.octave
  && Int.equal a.alteration b.alteration
;;

let compare a b =
  match Int.compare a.degree b.degree with
  | 0 -> (
      match Int.compare a.octave b.octave with
      | 0 -> Int.compare a.alteration b.alteration
      | nonzero -> nonzero)
  | nonzero -> nonzero
;;
