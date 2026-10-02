open! Base

type t =
  { degree : int
  ; alteration : Alteration.t
  }

let create ~degree ~alteration = { degree; alteration }
let natural ~degree = { degree; alteration = Alteration.Natural }
let degree t = t.degree
let alteration t = t.alteration
let diatonic = List.init 7 ~f:(fun i -> natural ~degree:(i + 1))

let equal a b =
  Int.equal a.degree b.degree && Alteration.equal a.alteration b.alteration
;;

let compare a b =
  match Int.compare a.degree b.degree with
  | 0 -> Alteration.compare a.alteration b.alteration
  | nonzero -> nonzero
;;

let label t = Alteration.to_string t.alteration ^ Int.to_string t.degree

let pitch_class ~root ~mode t =
  let base = List.nth_exn (Mode.pitch_classes mode ~root) (t.degree - 1) in
  Pitch_class.add base (Alteration.semitones t.alteration)
;;
