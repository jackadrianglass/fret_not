open! Base

type t =
  { start_x : float
  ; cell_width : float
  ; cell_height : float
  ; cell_gap : float
  }

val cell_x : t -> slot_index:int -> float
val cell_center_x : t -> slot_index:int -> float
val cell_center_y : t -> float

val canvas_width : t -> slot_count:int -> float
(** The strip's own width, without [start_x]: [slot_count] cells separated by
    gaps, no trailing gap. *)

val canvas_height : t -> float

val slot_at_point : t -> slot_count:int -> x:float -> y:float -> int option
(** Which cell a strip-relative point falls in; [None] in a gap, off the strip,
    or when there are no slots. *)
