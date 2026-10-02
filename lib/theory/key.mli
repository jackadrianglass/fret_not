open! Base

type quality =
  | Major
  | Minor

type t

val create : tonic:Spelled_pitch.t -> quality:quality -> t
val tonic : t -> Spelled_pitch.t
val quality : t -> quality
val tonic_pitch_class : t -> Pitch_class.t
val parent_major_root : t -> Spelled_pitch.t
val mode_of_quality : quality -> Mode.t
val mode_root : t -> Mode.t -> Spelled_pitch.t
val modes : t -> (Mode.t * Pitch_class.t list) list
val mode_with_root : t -> Pitch_class.t -> Mode.t option
