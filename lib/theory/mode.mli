open! Base

type t =
  | Ionian
  | Dorian
  | Phrygian
  | Lydian
  | Mixolydian
  | Aeolian
  | Locrian

val all : t list
val name : t -> string

val root_offset_semitones : t -> int

val degrees : t -> root:Pitch_class.t -> Scale_degree.t list
