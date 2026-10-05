open! Base
open Fret_not

let layout =
  { Chunk_layout.start_x = 10.
  ; cell_width = 20.
  ; cell_height = 30.
  ; cell_gap = 5.
  }
;;

let cells_advance_by_width_and_gap () =
  Alcotest.(check (float 0.001))
    "cell 0 starts at start_x" 10.
    (Chunk_layout.cell_x layout ~slot_index:0);
  Alcotest.(check (float 0.001))
    "cell 1 follows the gap" 35.
    (Chunk_layout.cell_x layout ~slot_index:1);
  Alcotest.(check (float 0.001))
    "cell 2 follows the gap" 60.
    (Chunk_layout.cell_x layout ~slot_index:2)
;;

let cell_centers_are_cell_midpoints () =
  Alcotest.(check (float 0.001))
    "center of cell 0" 20.
    (Chunk_layout.cell_center_x layout ~slot_index:0);
  Alcotest.(check (float 0.001))
    "center of cell 1" 45.
    (Chunk_layout.cell_center_x layout ~slot_index:1);
  Alcotest.(check (float 0.001))
    "vertical center" 15.
    (Chunk_layout.cell_center_y layout)
;;

let canvas_width_counts_no_trailing_gap () =
  Alcotest.(check (float 0.001))
    "three cells" 70.
    (Chunk_layout.canvas_width layout ~slot_count:3);
  Alcotest.(check (float 0.001))
    "one cell" 20.
    (Chunk_layout.canvas_width layout ~slot_count:1);
  Alcotest.(check (float 0.001))
    "no cells" 0.
    (Chunk_layout.canvas_width layout ~slot_count:0)
;;

let canvas_height_is_cell_height () =
  Alcotest.(check (float 0.001))
    "cell height" 30.
    (Chunk_layout.canvas_height layout)
;;

let slot_at_point_hits_cells_only () =
  let hit x y = Chunk_layout.slot_at_point layout ~slot_count:3 ~x ~y in
  Alcotest.(check (option int)) "inside cell 1" (Some 1) (hit 36. 0.);
  Alcotest.(check (option int)) "top edge of cell 2" (Some 2) (hit 60. 30.);
  Alcotest.(check (option int)) "gap between cells" None (hit 32. 15.);
  Alcotest.(check (option int)) "past the last cell" None (hit 85. 15.);
  Alcotest.(check (option int)) "above the strip" None (hit 40. 31.);
  Alcotest.(check (option int)) "left of the strip" None (hit 5. 15.)
;;

let empty_strip_never_hits () =
  Alcotest.(check (option int))
    "no slots means no hit" None
    (Chunk_layout.slot_at_point layout ~slot_count:0 ~x:40. ~y:15.)
;;

let tests =
  [ Alcotest.test_case "cells advance by width and gap" `Quick
      cells_advance_by_width_and_gap
  ; Alcotest.test_case "cell centers are cell midpoints" `Quick
      cell_centers_are_cell_midpoints
  ; Alcotest.test_case "canvas width counts no trailing gap" `Quick
      canvas_width_counts_no_trailing_gap
  ; Alcotest.test_case "canvas height is cell height" `Quick
      canvas_height_is_cell_height
  ; Alcotest.test_case "slot_at_point hits cells only" `Quick
      slot_at_point_hits_cells_only
  ; Alcotest.test_case "empty strip never hits" `Quick empty_strip_never_hits
  ]
;;
