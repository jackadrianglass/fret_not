open! Base

type t =
  { degree : int
  ; alteration : Alteration.t
  }
[@@deriving eq, ord]

val create : degree:int -> alteration:Alteration.t -> t
val natural : degree:int -> t
val degree : t -> int
val alteration : t -> Alteration.t
val diatonic : t list
val label : t -> string
val pitch_class : root:Pitch_class.t -> mode:Mode.t -> t -> Pitch_class.t

val spelled : root:Spelled_pitch.t -> mode:Mode.t -> t -> Spelled_pitch.t
(** The degree as a spelled pitch: the letter is the root's letter stepped
    [degree - 1] letters up; the existing alteration field is the accidental.
    Raises when the spelling would need more than a double sharp/flat. *)
