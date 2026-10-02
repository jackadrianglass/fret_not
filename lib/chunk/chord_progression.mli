open! Base

type t = int list

val equal : t -> t -> bool

val apply_to_chunk : t -> Chunk.t -> Chunk.t list
