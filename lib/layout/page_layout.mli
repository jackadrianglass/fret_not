open! Base

type t =
  { control_bar_height : float
  ; chunk_area_height : float
  ; tab_area_height : float
  ; fretboard_height : float
  }

val content_height : t -> float
(** The minimum window height: bar + taller page + fretboard. Below it the
    fretboard would overlap the page content. *)

val content_top_y : t -> float
(** Where the current page's own content starts, under the control bar. *)

val fretboard_top_y : t -> window_height:float -> float
(** Anchored to the window's bottom: the same place on both pages. *)
