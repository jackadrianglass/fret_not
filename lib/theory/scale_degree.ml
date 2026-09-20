open! Base

type t =
  { degree : int
  ; pitch_class : Pitch_class.t
  }

let equal a b =
  Int.equal a.degree b.degree && Pitch_class.equal a.pitch_class b.pitch_class
;;

let compare a b =
  match Int.compare a.degree b.degree with
  | 0 -> Pitch_class.compare a.pitch_class b.pitch_class
  | nonzero -> nonzero
;;
