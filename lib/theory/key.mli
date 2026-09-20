open! Base

type quality =
  | Major
  | Minor

type t

val create : tonic:Pitch_class.t -> quality:quality -> t
val tonic : t -> Pitch_class.t
val quality : t -> quality

val mode_root : t -> Mode.t -> Pitch_class.t
(** The pitch class this mode's own tonic sits on within this key (e.g. Dorian's
    root in the key of C major is D). *)

val modes : t -> (Mode.t * Scale_degree.t list) list
(** All 7 diatonic modes implied by this key, each paired with its own scale
    degrees. A major key and its relative minor produce the same 7 (mode,
    degrees) pairs. *)
