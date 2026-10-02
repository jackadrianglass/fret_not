open! Base

type t =
  { scale_degree : Scale_degree.t
  ; octave : int
  }

let create ~scale_degree ~octave = { scale_degree; octave }

let natural ~degree ~octave =
  create ~scale_degree:(Scale_degree.natural ~degree) ~octave
;;

let scale_degree t = t.scale_degree
let degree t = t.scale_degree.degree
let alteration t = t.scale_degree.alteration
let octave t = t.octave

let equal a b =
  Scale_degree.equal a.scale_degree b.scale_degree
  && Int.equal a.octave b.octave
;;

let compare a b =
  match Scale_degree.compare a.scale_degree b.scale_degree with
  | 0 -> Int.compare a.octave b.octave
  | nonzero -> nonzero
;;
