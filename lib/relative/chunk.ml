open! Base

type slot =
  | Rest
  | Note of Degree_reference.t
[@@deriving eq]

type t = slot list [@@deriving eq]

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
    | Rest -> Rest
    | Note dr -> Note (reframe_degree_reference dr ~root_degree))
;;

let apply_progression t (root_degrees : int list) =
  List.map root_degrees ~f:(fun root_degree -> reframe t ~root_degree)
;;
