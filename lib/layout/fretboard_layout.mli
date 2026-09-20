open! Base

type t =
  { canvas_width : float
  ; canvas_height : float
  ; margin : float
  ; string_count : int
  ; fret_count : int
  }

val string_y : t -> string_index:int -> float
(** string_index 0 (lowest string) is at the bottom of the canvas, matching tab
    notation's top-to-bottom string order. *)

val fret_line_x : t -> fret:int -> float
(** The vertical grid line marking the boundary before this fret; fret 0 is the
    nut. Ranges over 0..fret_count. *)

val fret_center_x : t -> fret:int -> float
(** Where fret's own content (a highlight, a fret-number label) centers
    horizontally — the nut line for fret 0, or mid-cell between its two bounding
    fret lines otherwise. *)

val position_point : t -> Fretboard_position.t -> float * float
(** Where to center a highlight for this position — on the nut line for an open
    string (fret 0), or mid-cell between its two bounding fret lines otherwise.
*)
