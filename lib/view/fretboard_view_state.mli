open! Base

type label_mode =
  | Degree_number
  | Note_name

type scale =
  | Diatonic
  | Pentatonic
  | Arpeggio
[@@deriving eq]

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

val tonics : Spelled_pitch.t list
(** One per tonic dropdown entry, indexed by [tonic_index]. *)

val quality_of_index : int -> Key.quality
val label_mode_of_index : int -> label_mode
val scale_of_index : int -> scale
val mode_of_quality : Key.quality -> Mode.t
val key : t -> Key.t
val mode : t -> Mode.t

val scale_degrees : t -> Scale_degree.t list
(** The scale's degree subset; the same list drives highlighting and shapes. *)

val notes_per_string : t -> int
val scale_mode : t -> Mode.t
val scale_root : t -> Spelled_pitch.t

val highlighted_positions :
  t -> config:Fretboard_view_config.t -> Fretboard_position.t list

val position_shapes :
  t -> config:Fretboard_view_config.t -> Fretboard_position.t list list

val selected_position :
  t -> config:Fretboard_view_config.t -> Fretboard_position.t list option

val position_options : t -> config:Fretboard_view_config.t -> string
(** Derived from the actual shape list, so it never advertises a shape the
    instrument can't play. *)

val position_label_text :
     root:Spelled_pitch.t
  -> mode:Mode.t
  -> label_mode:label_mode
  -> Scale_degree.t
  -> string
