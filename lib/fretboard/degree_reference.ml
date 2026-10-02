open! Base

type t =
  { scale_degree : Scale_degree.t
  ; octave : int
  }

let create ~scale_degree ~octave = { scale_degree; octave }

let natural ~degree ~octave =
  create ~scale_degree:(Scale_degree.natural ~degree) ~octave
;;

let scale_degree t = t.scale_degree
let degree t = t.scale_degree.degree
let alteration t = t.scale_degree.alteration
let octave t = t.octave

let equal a b =
  Scale_degree.equal a.scale_degree b.scale_degree
  && Int.equal a.octave b.octave
;;

let compare a b =
  match Scale_degree.compare a.scale_degree b.scale_degree with
  | 0 -> Int.compare a.octave b.octave
  | nonzero -> nonzero
;;

let interval ~root ~mode (dr : t) =
  let pitch_class = Scale_degree.pitch_class ~root ~mode dr.scale_degree in
  ((Pitch_class.to_int pitch_class - Pitch_class.to_int root + 12) % 12)
  + (12 * dr.octave)
;;

let note ~root:(root_note : Note.t) ~mode (dr : t) =
  let spelled =
    Scale_degree.spelled
      ~root:(Note.spelled_pitch root_note)
      ~mode dr.scale_degree
  in
  let base =
    (Pitch_class.to_int (Spelled_pitch.pitch_class spelled)
    - Pitch_class.to_int (Note.pitch_class root_note)
    + 12)
    % 12
  in
  let sounding = Note.semitone root_note + base + (12 * dr.octave) in
  Note.create ~spelled_pitch:spelled
    ~octave:(((sounding - Spelled_pitch.semitone spelled) / 12) - 1)
;;
