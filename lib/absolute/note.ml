open! Base

type t =
  { spelled_pitch : Spelled_pitch.t
  ; octave : int
  }
[@@deriving eq]

let create ~spelled_pitch ~octave = { spelled_pitch; octave }

let of_parts ~letter ~alteration ~octave =
  create ~spelled_pitch:(Spelled_pitch.create ~letter ~alteration) ~octave
;;

let natural ~letter ~octave =
  create ~spelled_pitch:(Spelled_pitch.natural letter) ~octave
;;

let spelled_pitch t = t.spelled_pitch
let octave t = t.octave
let letter t = Spelled_pitch.letter t.spelled_pitch
let alteration t = Spelled_pitch.alteration t.spelled_pitch
let semitone t = (12 * (t.octave + 1)) + Spelled_pitch.semitone t.spelled_pitch
let pitch_class t = Pitch_class.of_int (semitone t)

let to_string t =
  Spelled_pitch.to_string t.spelled_pitch ^ Int.to_string t.octave
;;

let shift_octave t n = { t with octave = t.octave + n }

(* Not derived: the order is by sounding semitone, not by the record
   fields; spelling only breaks ties. *)
let compare a b =
  match Int.compare (semitone a) (semitone b) with
  | 0 -> Spelled_pitch.compare a.spelled_pitch b.spelled_pitch
  | nonzero -> nonzero
;;
