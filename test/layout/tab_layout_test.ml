open! Base
open Fret_not

let layout : Tab_layout.t =
  { canvas_width = 1200.
  ; margin = 20.
  ; string_count = 6
  ; row_spacing = 24.
  ; note_count = 7
  }
;;

let canvas_height_is_margins_plus_spacing_between_strings () =
  Alcotest.(check (float 0.001))
    "2*margin + row_spacing*(string_count-1)" 160.
    (Tab_layout.canvas_height ~margin:20. ~string_count:6 ~row_spacing:24.)
;;

let lowest_string_sits_at_the_bottom_of_the_canvas () =
  Alcotest.(check (float 0.001))
    "string 0 at canvas_height - margin" 140.
    (Tab_layout.string_y layout ~string_index:0)
;;

let highest_string_sits_at_the_top_margin () =
  Alcotest.(check (float 0.001))
    "top string at margin" 20.
    (Tab_layout.string_y layout ~string_index:5)
;;

let notes_span_from_margin_to_margin () =
  Alcotest.(check (float 0.001))
    "note 0 at margin" 20.
    (Tab_layout.note_x layout ~note_index:0);
  Alcotest.(check (float 0.001))
    "last note at canvas_width - margin" 1180.
    (Tab_layout.note_x layout ~note_index:6)
;;

let notes_are_evenly_spaced () =
  let spacing = (1200. -. (2. *. 20.)) /. 6. in
  Alcotest.(check (float 0.001))
    "note 1 one spacing past note 0" (20. +. spacing)
    (Tab_layout.note_x layout ~note_index:1)
;;

let tests =
  [ Alcotest.test_case "canvas_height is margins plus spacing between strings"
      `Quick canvas_height_is_margins_plus_spacing_between_strings
  ; Alcotest.test_case "lowest string at the bottom" `Quick
      lowest_string_sits_at_the_bottom_of_the_canvas
  ; Alcotest.test_case "highest string at the top margin" `Quick
      highest_string_sits_at_the_top_margin
  ; Alcotest.test_case "notes span from margin to margin" `Quick
      notes_span_from_margin_to_margin
  ; Alcotest.test_case "notes are evenly spaced" `Quick notes_are_evenly_spaced
  ]
;;
