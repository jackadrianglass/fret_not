open! Base

type t =
  { canvas_width : float
  ; canvas_height : float
  ; margin : float
  ; string_count : int
  ; fret_count : int
  }

val string_y : t -> string_index:int -> float
val fret_line_x : t -> fret:int -> float
val fret_center_x : t -> fret:int -> float
val position_point : t -> Fretboard_position.t -> float * float
