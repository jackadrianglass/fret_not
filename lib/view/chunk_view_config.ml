open! Base

type t =
  { margin : float
  ; cell_width : float
  ; cell_height : float
  ; cell_gap : float
  ; cursor_outline_extra : float
  ; slot_font_size : int
  ; strip_gap : float
  ; inspector_control_height : float
  ; inspector_gap : float
  ; inspector_control_padding : float
  ; dropdown_arrow_width : float
  ; button_width : float
  ; panel_gap : float
  ; max_slots : int
  }

let panel_height (t : t) =
  t.cell_height +. t.strip_gap +. t.inspector_control_height
;;

let default =
  { margin = 26.
  ; cell_width = 44.
  ; cell_height = 44.
  ; cell_gap = 8.
  ; cursor_outline_extra = 3.
  ; slot_font_size = 18
  ; strip_gap = 12.
  ; inspector_control_height = 20.
  ; inspector_gap = 10.
  ; inspector_control_padding = 12.
  ; dropdown_arrow_width = 16.
  ; button_width = 24.
  ; panel_gap = 16.
  ; max_slots = 16
  }
;;
