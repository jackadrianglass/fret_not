open! Base

type slot =
  | Rest
  | Note of Degree_reference.t
[@@deriving eq]

(** A chunk's rhythm-bearing element: slot presence IS the rhythm; all slots
    equal duration (deliberate simplification). *)

type t = slot list [@@deriving eq]

(** A movable pattern: reframe rotates it onto any degree of the key. *)

val reframe : t -> root_degree:int -> t
(** Shifts every note's degree by [root_degree - 1], carrying the octave when
    the degree wraps past 7 — interval content is what's preserved. *)

val apply_progression : t -> int list -> t list
(** One reframed chunk per root degree, e.g. [1; 4; 5]. *)
