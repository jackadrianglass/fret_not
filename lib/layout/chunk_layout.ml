open! Base

type t =
  { start_x : float
  ; cell_width : float
  ; cell_height : float
  ; cell_gap : float
  }

let pitch t = t.cell_width +. t.cell_gap
let cell_x t ~slot_index = t.start_x +. (Float.of_int slot_index *. pitch t)
let cell_center_x t ~slot_index = cell_x t ~slot_index +. (t.cell_width /. 2.)
let cell_center_y t = t.cell_height /. 2.

let canvas_width t ~slot_count =
  (Float.of_int slot_count *. t.cell_width)
  +. (Float.of_int (Int.max 0 (slot_count - 1)) *. t.cell_gap)
;;

let canvas_height t = t.cell_height

let slot_at_point t ~slot_count ~x ~y =
  if slot_count = 0 then None
  else if Float.(y < 0. || y > t.cell_height) then None
  else if Float.(x < t.start_x) then None
  else
    let index = Int.of_float ((x -. t.start_x) /. pitch t) in
    if index >= slot_count then None
    else if Float.(x <= cell_x t ~slot_index:index +. t.cell_width) then
      Some index
    else None
;;
