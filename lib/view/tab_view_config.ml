open! Base

type t =
  { margin : float
  ; row_spacing : float
  ; string_label_font_size : int
  ; fret_number_font_size : int
  ; rule_gap : float
  ; rule_thickness : float
  }

let default =
  { margin = 26.
  ; row_spacing = 32.
  ; string_label_font_size = 16
  ; fret_number_font_size = 18
  ; rule_gap = 16.
  ; rule_thickness = 4.
  }
;;
