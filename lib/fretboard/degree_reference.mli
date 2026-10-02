open! Base

type t =
  { scale_degree : Scale_degree.t
  ; octave : int
  }

val create : scale_degree:Scale_degree.t -> octave:int -> t
val natural : degree:int -> octave:int -> t
val scale_degree : t -> Scale_degree.t
val degree : t -> int
val alteration : t -> Alteration.t
val octave : t -> int
val equal : t -> t -> bool
val compare : t -> t -> int

val interval : root:Pitch_class.t -> mode:Mode.t -> t -> int
(** Semitones above the tonic. The octave is tonic-anchored: octave 0 spans
    from the tonic up to (but not including) the octave above, so degree 1
    octave 0 IS the tonic and the flat 1 of octave 0 sounds a semitone below
    the octave above the tonic. *)

val note : root:Note.t -> mode:Mode.t -> t -> Note.t
(** Absolute spelled note; [root] supplies both the spelling anchor and the
    octave register. *)
