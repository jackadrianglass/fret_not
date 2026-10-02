open! Base

type t =
  { margin : float
  ; row_spacing : float
  ; string_label_font_size : int
  ; fret_number_font_size : int
  ; rule_gap : float
  ; rule_thickness : float
  }

val default : t
