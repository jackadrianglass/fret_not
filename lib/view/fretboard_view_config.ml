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
  ; dimmed_alpha : float
  ; dropdown_left_text_padding : int
  ; control_bar_x : float
  ; control_bar_y : float
  ; control_height : float
  ; control_gap : float
  ; dropdown_text_to_arrow_gap : int
  ; window_title : string
  ; target_fps : int
  }

let default =
  { canvas_width = 1560
  ; canvas_height = 360
  ; control_bar_height = 50
  ; margin = 65.
  ; instrument =
      Instrument.create_uniform ~tuning:Tuning.standard ~fret_count:22
  ; position_dot_radius = 14.
  ; root_halo_radius = 18.
  ; label_font_size = 14
  ; fret_number_font_size = 18
  ; fret_number_gap_below_lowest_string = 8
  ; dimmed_alpha = 0.3
  ; dropdown_left_text_padding = 6
  ; control_bar_x = 10.
  ; control_bar_y = 15.
  ; control_height = 20.
  ; control_gap = 10.
  ; dropdown_text_to_arrow_gap = 12
  ; window_title = "fret_not"
  ; target_fps = 60
  }
;;
