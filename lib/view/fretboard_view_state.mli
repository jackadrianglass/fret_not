open! Base

type label_mode =
  | Degree_number
  | Note_name

type scale =
  | Diatonic
  | Pentatonic
  | Arpeggio

type t =
  { tonic_index : int
  ; quality_index : int
  ; tonic_dropdown_open : bool
  ; quality_dropdown_open : bool
  ; label_mode_index : int
  ; label_mode_dropdown_open : bool
  ; scale_index : int
  ; scale_dropdown_open : bool
  ; position_index : int
  ; position_dropdown_open : bool
  }

val initial : t
val quality_of_index : int -> Key.quality
val label_mode_of_index : int -> label_mode
val scale_of_index : int -> scale
val mode_of_quality : Key.quality -> Mode.t
val key : t -> Key.t
val mode : t -> Mode.t
val scale_degrees : t -> Scale_degree.t list

val highlighted_positions :
  t -> config:Fretboard_view_config.t -> Fretboard_position.t list

val three_notes_per_string_positions :
  t -> config:Fretboard_view_config.t -> Fretboard_position.t list list

val two_notes_per_string_positions :
  t -> config:Fretboard_view_config.t -> Fretboard_position.t list list

val one_note_per_string_positions :
  t -> config:Fretboard_view_config.t -> Fretboard_position.t list list

val selected_position :
  t -> config:Fretboard_view_config.t -> Fretboard_position.t list option

val mode_names_from : Mode.t -> string list
val position_options : t -> string
val position_label_text : label_mode:label_mode -> Scale_degree.t -> string
