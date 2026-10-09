open! Base
open Fret_not

let x_positions_lays_out_left_to_right_with_gaps () =
  Alcotest.(check (list (float 0.001)))
    "start_x, then cumulative width+gap" [ 0.; 60.; 100. ]
    (Row_layout.x_positions ~start_x:0. ~gap:10. [ 50.; 30.; 20. ])
;;

let x_positions_respects_a_nonzero_start_x () =
  Alcotest.(check (list (float 0.001)))
    "first position is exactly start_x" [ 10.; 25. ]
    (Row_layout.x_positions ~start_x:10. ~gap:5. [ 10.; 10. ])
;;

let x_positions_from_right_ends_flush_at_end_x () =
  Alcotest.(check (list (float 0.001)))
    "last width ends at end_x, order preserved" [ 80.; 140.; 180. ]
    (Row_layout.x_positions_from_right ~end_x:200. ~gap:10. [ 50.; 30.; 20. ])
;;

let x_positions_from_right_respects_a_single_width () =
  Alcotest.(check (list (float 0.001)))
    "one width lands exactly at end_x" [ 175. ]
    (Row_layout.x_positions_from_right ~end_x:200. ~gap:10. [ 25. ])
;;

let dropdown_width_adds_padding_arrow_and_gap () =
  Alcotest.(check (float 0.001))
    "left_padding + widest + arrow_padding + gap"
    (Float.of_int (6 + 40 + 12 + 12))
    (Row_layout.dropdown_width ~left_text_padding:6 ~widest_option_text_width:40
       ~arrow_padding:12 ~text_to_arrow_gap:12)
;;

let tests =
  [ Alcotest.test_case "x_positions lays out left to right with gaps" `Quick
      x_positions_lays_out_left_to_right_with_gaps
  ; Alcotest.test_case "x_positions respects a nonzero start_x" `Quick
      x_positions_respects_a_nonzero_start_x
  ; Alcotest.test_case "x_positions_from_right ends flush at end_x" `Quick
      x_positions_from_right_ends_flush_at_end_x
  ; Alcotest.test_case "x_positions_from_right respects a single width" `Quick
      x_positions_from_right_respects_a_single_width
  ; Alcotest.test_case "dropdown_width adds padding, arrow, and gap" `Quick
      dropdown_width_adds_padding_arrow_and_gap
  ]
;;
