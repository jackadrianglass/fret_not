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
(** canvas_width and tuning intentionally aren't config fields here - the tab
    view shares those with Fretboard_view_config at composition time rather than
    storing its own possibly-diverging copy. *)
