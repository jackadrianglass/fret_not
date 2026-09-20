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
(** Semitone distance of this mode's tonic above the parent major scale's root
    (e.g. Dorian is 2 semitones above the major scale it's drawn from). *)

val degrees : t -> root:Pitch_class.t -> Scale_degree.t list
