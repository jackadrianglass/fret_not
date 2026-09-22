open! Base

type t = int list

let equal = List.equal Int.equal

let apply_to_chunk t chunk =
  List.map t ~f:(fun root_degree -> Chunk.reframe chunk ~root_degree)
;;
