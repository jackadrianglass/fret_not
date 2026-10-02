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
