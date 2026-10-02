open! Base

type t =
  { spelled_pitch : Spelled_pitch.t
  ; octave : int
  }

(** A note in scientific pitch notation: a spelled pitch plus an octave, e.g.
    C4, E2, Bb3. The octave is the SPN octave: it increments at B -> C, so B3
    sounds below C4, and B#3 sounds as C4's pitch class. *)

val create : spelled_pitch:Spelled_pitch.t -> octave:int -> t
val of_parts : letter:Letter.t -> alteration:Alteration.t -> octave:int -> t
val natural : letter:Letter.t -> octave:int -> t
val spelled_pitch : t -> Spelled_pitch.t
val octave : t -> int
val letter : t -> Letter.t
val alteration : t -> Alteration.t

val semitone : t -> int
(** Sounding semitones above C0; 12 * (octave + 1) + spelled semitone. *)

val pitch_class : t -> Pitch_class.t
val to_string : t -> string
val shift_octave : t -> int -> t

val equal : t -> t -> bool
(** Spelling-sensitive: C#4 <> Db4. *)

val compare : t -> t -> int
(** By sounding semitone, ties broken by spelling. *)
