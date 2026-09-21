open! Base

type t =
  { canvas_width : float
  ; margin : float
  ; string_count : int
  ; row_spacing : float
  ; note_count : int
  }

let canvas_height ~margin ~string_count ~row_spacing =
  (2. *. margin) +. (row_spacing *. Float.of_int (string_count - 1))
;;

let string_y t ~string_index =
  canvas_height ~margin:t.margin ~string_count:t.string_count
    ~row_spacing:t.row_spacing
  -. t.margin
  -. (Float.of_int string_index *. t.row_spacing)
;;

let note_x t ~note_index =
  let spacing =
    (t.canvas_width -. (2. *. t.margin)) /. Float.of_int (t.note_count - 1)
  in
  t.margin +. (Float.of_int note_index *. spacing)
;;
