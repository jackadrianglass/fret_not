open! Base

type t =
  { canvas_width : float
  ; margin : float
  ; string_count : int
  ; row_spacing : float
  ; note_count : int
  }

val canvas_height :
  margin:float -> string_count:int -> row_spacing:float -> float
(** margin on both ends plus row_spacing between each string line. *)

val string_y : t -> string_index:int -> float
(** string_index 0 (lowest string) is at the bottom of the canvas, matching
    Fretboard_layout's convention. *)

val note_x : t -> note_index:int -> float
(** note_index 0..note_count-1 evenly spaced between the margins. Assumes
    note_count >= 2 (always true for the tab-able scale sequences, which have at
    least 7 notes). *)
