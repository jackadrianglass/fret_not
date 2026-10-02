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
val pitch_classes : t -> root:Pitch_class.t -> Pitch_class.t list
