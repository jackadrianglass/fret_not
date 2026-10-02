open! Base

type t =
  { letter : Letter.t
  ; alteration : Alteration.t
  }

let create ~letter ~alteration = { letter; alteration }
let natural letter = { letter; alteration = Alteration.Natural }
let letter t = t.letter
let alteration t = t.alteration

let semitone t =
  Pitch_class.to_int (Letter.to_pitch_class t.letter)
  + Alteration.semitones t.alteration
;;

let pitch_class t = Pitch_class.of_int (semitone t)
let to_string t = Letter.to_string t.letter ^ Alteration.to_string t.alteration

let alteration_for ~letter ~pitch_class =
  let natural_pitch_class = Letter.to_pitch_class letter in
  let delta =
    (Pitch_class.to_int pitch_class
    - Pitch_class.to_int natural_pitch_class
    + 12)
    % 12
  in
  let delta = if delta > 6 then delta - 12 else delta in
  Alteration.of_semitones delta
;;

let of_pitch_class_exn ~letter ~pitch_class =
  match alteration_for ~letter ~pitch_class with
  | Some alteration -> create ~letter ~alteration
  | None ->
      raise
        (Invalid_argument
           (Printf.sprintf "no accidental makes %s sound as %s"
              (Letter.to_string letter)
              (Pitch_class.to_string pitch_class)))
;;

let equal a b =
  Letter.equal a.letter b.letter && Alteration.equal a.alteration b.alteration
;;

let compare a b =
  match Letter.compare a.letter b.letter with
  | 0 -> Alteration.compare a.alteration b.alteration
  | nonzero -> nonzero
;;
