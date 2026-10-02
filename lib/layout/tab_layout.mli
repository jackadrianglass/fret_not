open! Base

type t =
  { canvas_width : float
  ; margin : float
  ; string_count : int
  ; row_spacing : float
  ; note_count : int
  }

val canvas_height :
  margin:float -> string_count:int -> row_spacing:float -> float

val string_y : t -> string_index:int -> float
val note_x : t -> note_index:int -> float
