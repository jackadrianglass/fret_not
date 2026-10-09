open! Base

type label_mode =
  | Degree_number
  | Note_name

type page =
  | Chunk_editing
  | Tab_viewing
[@@deriving eq]

type t =
  { page_index : int
  ; tonic_index : int
  ; quality_index : int
  ; tonic_dropdown_open : bool
  ; quality_dropdown_open : bool
  ; label_mode_index : int
  ; label_mode_dropdown_open : bool
  }

val initial : t

val tonics : Spelled_pitch.t list
(** One per tonic dropdown entry, indexed by [tonic_index]. *)

val page_of_index : int -> page
val quality_of_index : int -> Key.quality
val label_mode_of_index : int -> label_mode
val mode_of_quality : Key.quality -> Mode.t
val key : t -> Key.t
val mode : t -> Mode.t

val highlighted_positions :
  t -> config:Fretboard_view_config.t -> Fretboard_position.t list
(** Every in-key position on the instrument. *)

val position_label_text :
     root:Spelled_pitch.t
  -> mode:Mode.t
  -> label_mode:label_mode
  -> Scale_degree.t
  -> string
