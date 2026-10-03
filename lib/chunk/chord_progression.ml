open! Base

type t = int list [@@deriving eq]

let apply_to_chunk t chunk =
  List.map t ~f:(fun root_degree -> Chunk.reframe chunk ~root_degree)
;;
