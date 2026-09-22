open! Base

type t = int list
(** Scale-degree roots, 1-based, one per step - e.g. [ 1; 4; 5 ] for a I-IV-V
    progression. Its own movable object, not a Song attribute: quality is
    inferred diatonically from whatever Key+Mode it's eventually paired with, no
    explicit per-step quality override yet. *)

val equal : t -> t -> bool

val apply_to_chunk : t -> Chunk.t -> Chunk.t list
(** One reframed Chunk per progression step, in order - see Chunk.reframe. *)
