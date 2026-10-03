open! Base

type t = Slot.t list [@@deriving eq]

let reframe_degree_reference (dr : Degree_reference.t) ~root_degree =
  let raw_degree = Degree_reference.degree dr - 1 + (root_degree - 1) in
  { Degree_reference.scale_degree =
      Scale_degree.create
        ~degree:((raw_degree % 7) + 1)
        ~alteration:(Degree_reference.alteration dr)
  ; octave = Degree_reference.octave dr + (raw_degree / 7)
  }
;;

let reframe t ~root_degree =
  List.map t ~f:(function
    | Slot.Rest -> Slot.Rest
    | Slot.Note dr -> Slot.Note (reframe_degree_reference dr ~root_degree))
;;
