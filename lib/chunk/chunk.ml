open! Base

type t = Slot.t list

let equal = List.equal Slot.equal

let reframe_degree_reference (dr : Degree_reference.t) ~root_degree =
  let raw_degree = dr.degree - 1 + (root_degree - 1) in
  { Degree_reference.degree = (raw_degree % 7) + 1
  ; octave = dr.octave + (raw_degree / 7)
  ; alteration = dr.alteration
  }
;;

let reframe t ~root_degree =
  List.map t ~f:(function
    | Slot.Rest -> Slot.Rest
    | Slot.Note dr -> Slot.Note (reframe_degree_reference dr ~root_degree))
;;
