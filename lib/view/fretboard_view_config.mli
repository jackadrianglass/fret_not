open! Base

type t =
  { canvas_width : int
  ; canvas_height : int
  ; control_bar_height : int
  ; margin : float
  ; instrument : Instrument.t
  ; position_dot_radius : float
  ; root_halo_radius : float
  ; label_font_size : int
  ; fret_number_font_size : int
  ; fret_number_gap_below_lowest_string : int
  ; dropdown_left_text_padding : int
  ; control_bar_x : float
  ; control_bar_y : float
  ; control_height : float
  ; control_gap : float
  ; dropdown_text_to_arrow_gap : int
  ; window_title : string
  ; target_fps : int
  ; window_min_width : int
  ; control_bar_tint_alpha : float
  }

val default : t
(** Reproduces the fretboard view's original hardcoded presentation values. *)
