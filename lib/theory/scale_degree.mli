open! Base

type t =
  { degree : int
  ; alteration : Alteration.t
  }

val create : degree:int -> alteration:Alteration.t -> t
val natural : degree:int -> t
val degree : t -> int
val alteration : t -> Alteration.t
val diatonic : t list
val equal : t -> t -> bool
val compare : t -> t -> int
val label : t -> string
val pitch_class : root:Pitch_class.t -> mode:Mode.t -> t -> Pitch_class.t
