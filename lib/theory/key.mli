open! Base

type quality =
  | Major
  | Minor

type t

val create : tonic:Spelled_pitch.t -> quality:quality -> t
val tonic : t -> Spelled_pitch.t
val quality : t -> quality
val tonic_pitch_class : t -> Pitch_class.t
val mode_of_quality : quality -> Mode.t
val mode_root : t -> Mode.t -> Spelled_pitch.t
val mode_pitch_classes : t -> Mode.t -> Pitch_class.t list
