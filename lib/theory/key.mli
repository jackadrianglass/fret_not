open! Base

type quality =
  | Major
  | Minor

type t

val create : tonic:Pitch_class.t -> quality:quality -> t
val tonic : t -> Pitch_class.t
val quality : t -> quality

val mode_root : t -> Mode.t -> Pitch_class.t

val modes : t -> (Mode.t * Scale_degree.t list) list

val mode_with_root : t -> Pitch_class.t -> Mode.t option
