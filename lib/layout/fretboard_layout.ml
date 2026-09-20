open! Base

type t =
  { canvas_width : float
  ; canvas_height : float
  ; margin : float
  ; string_count : int
  ; fret_count : int
  }

let string_spacing t =
  (t.canvas_height -. (2. *. t.margin)) /. Float.of_int (t.string_count - 1)
;;

let string_y t ~string_index =
  t.canvas_height -. t.margin -. (Float.of_int string_index *. string_spacing t)
;;

let fret_spacing t =
  (t.canvas_width -. (2. *. t.margin)) /. Float.of_int t.fret_count
;;

let fret_line_x t ~fret = t.margin +. (Float.of_int fret *. fret_spacing t)

let fret_center_x t ~fret =
  if fret = 0 then fret_line_x t ~fret:0
  else (fret_line_x t ~fret:(fret - 1) +. fret_line_x t ~fret) /. 2.
;;

let position_point t (position : Fretboard_position.t) =
  ( fret_center_x t ~fret:position.fret
  , string_y t ~string_index:position.string_index )
;;
