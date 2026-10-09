open! Base

type t =
  { control_bar_height : float
  ; chunk_area_height : float
  ; tab_area_height : float
  ; fretboard_height : float
  }

let page_area_height t = Float.max t.chunk_area_height t.tab_area_height

(* The taller page fixes the minimum: below this height the fretboard would
   overlap the page content. *)
let content_height t =
  t.control_bar_height +. page_area_height t +. t.fretboard_height
;;

let content_top_y t = t.control_bar_height

(* Anchored to the window's bottom, so extra window height lands between the
   page content and the fretboard; independent of the page areas, so a page
   switch never moves it. *)
let fretboard_top_y t ~window_height = window_height -. t.fretboard_height
