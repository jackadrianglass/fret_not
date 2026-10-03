open! Base

let distance (a : Fretboard_position.t) (b : Fretboard_position.t) =
  let reach = Reach.between a b in
  reach.fret_span + reach.string_span
;;

let root_semitone ~key = Note.semitone (Fretboard.root_note ~key)

let base_interval ~key ~mode (dr : Degree_reference.t) =
  Degree_reference.interval
    ~root:(Key.tonic_pitch_class key)
    ~mode { dr with octave = 0 }
;;

(* Floor/ceil with the non-negative remainder, so negative windows (a degree
   whose first in-range occurrence is below the anchor register) divide
   correctly. *)
let floor_div a b =
  let remainder = Int.rem (Int.rem a b + b) b in
  (a - remainder) / b
;;

let ceil_div a b = -(floor_div (-a) b)

let first_note_octaves ~instrument ~key ~mode (dr : Degree_reference.t) =
  let lowest, highest = Instrument.range instrument in
  let base = base_interval ~key ~mode dr in
  let root = root_semitone ~key in
  let octave_lo = ceil_div (lowest - root - base) 12 in
  let octave_hi = floor_div (highest - root - base) 12 in
  if octave_lo > octave_hi then [] else List.range octave_lo (octave_hi + 1)
;;

let first_note_candidates ~key ~mode ~instrument
    ~(start_anchor : Fretboard_position.t) ~max_fret_distance
    (dr : Degree_reference.t) =
  List.concat_map (first_note_octaves ~instrument ~key ~mode dr)
    ~f:(fun octave ->
      Fretboard.degree_positions ~instrument ~key ~mode { dr with octave })
  |> List.filter ~f:(fun (p : Fretboard_position.t) ->
      Int.abs (p.fret - start_anchor.fret) <= max_fret_distance)
;;

let candidates_for_semitone ~instrument ~(near : Fretboard_position.t)
    ~max_fret_distance target_semitone =
  Fretboard.positions_of_semitone ~instrument ~target_semitone
  |> List.filter ~f:(fun (p : Fretboard_position.t) ->
      Int.abs (p.fret - near.fret) <= max_fret_distance)
;;

let rec chain ~key ~mode ~instrument ~max_fret_distance ~offset anchor =
  function
  | [] -> [ [] ]
  | dr :: rest ->
      let target =
        Note.semitone (Fretboard.degree_note ~key ~mode dr) + offset
      in
      candidates_for_semitone ~instrument ~near:anchor ~max_fret_distance target
      |> List.concat_map ~f:(fun position ->
          chain ~key ~mode ~instrument ~max_fret_distance ~offset position rest
          |> List.map ~f:(fun tail -> position :: tail))
;;

let total_distance anchor shape =
  let _, total =
    List.fold shape ~init:(anchor, 0) ~f:(fun (prev, acc) pos ->
        (pos, acc + distance prev pos))
  in
  total
;;

let positions ~key ~mode ~instrument ~start_anchor ~max_fret_distance = function
  | [] -> [ [] ]
  | first :: rest ->
      first_note_candidates ~key ~mode ~instrument ~start_anchor
        ~max_fret_distance first
      |> List.concat_map ~f:(fun (position : Fretboard_position.t) ->
          let offset =
            Instrument.semitone_at instrument position
            - Note.semitone (Fretboard.degree_note ~key ~mode first)
          in
          chain ~key ~mode ~instrument ~max_fret_distance ~offset position rest
          |> List.map ~f:(fun tail -> position :: tail))
      |> List.sort ~compare:(fun a b ->
          Int.compare
            (total_distance start_anchor a)
            (total_distance start_anchor b))
;;
