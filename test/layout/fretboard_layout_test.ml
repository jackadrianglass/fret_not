open! Base
open Fret_not

let layout : Fretboard_layout.t =
  { canvas_width = 1200.
  ; canvas_height = 400.
  ; margin = 50.
  ; string_count = 6
  ; fret_count = 12
  }
;;

let lowest_string_sits_at_the_bottom_of_the_canvas () =
  Alcotest.(check (float 0.001))
    "string 0 at canvas_height - margin" 350.
    (Fretboard_layout.string_y layout ~string_index:0)
;;

let highest_string_sits_at_the_top_margin () =
  Alcotest.(check (float 0.001))
    "top string at margin" 50.
    (Fretboard_layout.string_y layout ~string_index:5)
;;

let fret_lines_span_from_margin_to_margin () =
  Alcotest.(check (float 0.001))
    "fret 0 at margin" 50.
    (Fretboard_layout.fret_line_x layout ~fret:0);
  Alcotest.(check (float 0.001))
    "fret 12 at canvas_width - margin" 1150.
    (Fretboard_layout.fret_line_x layout ~fret:12)
;;

let fret_zero_centers_on_the_nut () =
  Alcotest.(check (float 0.001))
    "fret 0 centers at the nut" 50.
    (Fretboard_layout.fret_center_x layout ~fret:0)
;;

let fret_centers_mid_cell () =
  let fret_spacing = (1200. -. (2. *. 50.)) /. 12. in
  Alcotest.(check (float 0.001))
    "fret 1 centers mid-cell"
    (50. +. (fret_spacing /. 2.))
    (Fretboard_layout.fret_center_x layout ~fret:1)
;;

let open_string_highlight_sits_on_the_nut () =
  let x, _ =
    Fretboard_layout.position_point layout { string_index = 0; fret = 0 }
  in
  Alcotest.(check (float 0.001)) "open string at fret 0's line" 50. x
;;

let fretted_highlight_sits_mid_cell () =
  let x, _ =
    Fretboard_layout.position_point layout { string_index = 0; fret = 1 }
  in
  let fret_spacing = (1200. -. (2. *. 50.)) /. 12. in
  Alcotest.(check (float 0.001))
    "fret 1 midpoint"
    (50. +. (fret_spacing /. 2.))
    x
;;

let tests =
  [ Alcotest.test_case "lowest string at the bottom" `Quick
      lowest_string_sits_at_the_bottom_of_the_canvas
  ; Alcotest.test_case "highest string at the top margin" `Quick
      highest_string_sits_at_the_top_margin
  ; Alcotest.test_case "fret lines span margin to margin" `Quick
      fret_lines_span_from_margin_to_margin
  ; Alcotest.test_case "fret 0 centers on the nut" `Quick
      fret_zero_centers_on_the_nut
  ; Alcotest.test_case "fret centers mid-cell" `Quick fret_centers_mid_cell
  ; Alcotest.test_case "open string highlight on the nut" `Quick
      open_string_highlight_sits_on_the_nut
  ; Alcotest.test_case "fretted highlight sits mid-cell" `Quick
      fretted_highlight_sits_mid_cell
  ]
;;
