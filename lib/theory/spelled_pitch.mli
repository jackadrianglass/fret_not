open! Base

type t =
  { letter : Letter.t
  ; alteration : Alteration.t
  }
[@@deriving eq, ord]

val create : letter:Letter.t -> alteration:Alteration.t -> t
val natural : Letter.t -> t
val letter : t -> Letter.t
val alteration : t -> Alteration.t

val semitone : t -> int
(** Sounding semitones relative to the same letter's natural, NOT folded into an
    octave: Cb = -1, B# = 12. This is what lets Note handle the B/C octave wrap
    correctly. *)

val pitch_class : t -> Pitch_class.t

val alteration_for :
  letter:Letter.t -> pitch_class:Pitch_class.t -> Alteration.t option
(** The accidental that makes [letter] sound as [pitch_class], if one exists
    within double sharp/flat. *)

val of_pitch_class_exn : letter:Letter.t -> pitch_class:Pitch_class.t -> t
val to_string : t -> string
