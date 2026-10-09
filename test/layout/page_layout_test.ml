open! Base
open Fret_not

let layout =
  { Page_layout.control_bar_height = 50.
  ; chunk_area_height = 92.
  ; tab_area_height = 244.
  ; fretboard_height = 360.
  }
;;

let content_height_is_the_minimum_window_height () =
  Alcotest.(check (float 0.001))
    "bar + max(chunk, tab) + fretboard"
    (50. +. 244. +. 360.)
    (Page_layout.content_height layout)
;;

let fretboard_top_y_anchors_to_the_window_bottom () =
  Alcotest.(check (float 0.001))
    "window 800 tall" (800. -. 360.)
    (Page_layout.fretboard_top_y layout ~window_height:800.);
  Alcotest.(check (float 0.001))
    "window 1200 tall" (1200. -. 360.)
    (Page_layout.fretboard_top_y layout ~window_height:1200.)
;;

let fretboard_top_y_ignores_the_page_areas () =
  Alcotest.(check (float 0.001))
    "a taller chunk page lands the same" 640.
    (Page_layout.fretboard_top_y
       { layout with Page_layout.chunk_area_height = 300. }
       ~window_height:1000.)
;;

let content_top_y_sits_under_the_control_bar () =
  Alcotest.(check (float 0.001))
    "content starts at the bar height" 50.
    (Page_layout.content_top_y layout)
;;

let tests =
  [ Alcotest.test_case "content_height is the minimum window height" `Quick
      content_height_is_the_minimum_window_height
  ; Alcotest.test_case "fretboard_top_y anchors to the window bottom" `Quick
      fretboard_top_y_anchors_to_the_window_bottom
  ; Alcotest.test_case "fretboard_top_y ignores the page areas" `Quick
      fretboard_top_y_ignores_the_page_areas
  ; Alcotest.test_case "content_top_y sits under the control bar" `Quick
      content_top_y_sits_under_the_control_bar
  ]
;;
