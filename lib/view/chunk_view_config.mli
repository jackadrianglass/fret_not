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

val panel_height : t -> float
(** The editor's drawn height: strip plus gap plus inspector row. [panel_gap] is
    the breathing room below the editor before the next view. *)

val default : t
