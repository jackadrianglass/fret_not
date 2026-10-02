open! Base

let distance (a : Fretboard_position.t) (b : Fretboard_position.t) =
  let reach = Reach.between a b in
  reach.fret_span + reach.string_span
;;

let absolute_semitone ~key ~mode (dr : Degree_reference.t) =
  Note.semitone (Fretboard.degree_note ~key ~mode dr)
;;

let octave_sweep = List.range (-4) 5

let first_note_candidates ~key ~mode ~instrument
    ~(start_anchor : Fretboard_position.t) ~max_fret_distance
    (dr : Degree_reference.t) =
  List.concat_map octave_sweep ~f:(fun octave ->
      Fretboard.degree_positions ~instrument ~key ~mode { dr with octave })
  |> List.filter ~f:(fun (p : Fretboard_position.t) ->
      Int.abs (p.fret - start_anchor.fret) <= max_fret_distance)
;;

let candidates_for_semitone ~instrument ~(near : Fretboard_position.t)
    ~max_fret_distance target_semitone =
  List.init (Instrument.string_count instrument) ~f:Fn.id
  |> List.filter_map ~f:(fun string_index ->
      let fret =
        target_semitone
        - Tuning.open_semitone (Instrument.tuning instrument) ~string_index
      in
      let position = { Fretboard_position.string_index; fret } in
      if
        Instrument.playable instrument position
        && Int.abs (fret - near.fret) <= max_fret_distance
      then Some position
      else None)
;;

let rec chain ~key ~mode ~instrument ~max_fret_distance ~offset anchor =
  function
  | [] -> [ [] ]
  | dr :: rest ->
      let target = absolute_semitone ~key ~mode dr + offset in
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
            - absolute_semitone ~key ~mode first
          in
          chain ~key ~mode ~instrument ~max_fret_distance ~offset position rest
          |> List.map ~f:(fun tail -> position :: tail))
      |> List.sort ~compare:(fun a b ->
          Int.compare
            (total_distance start_anchor a)
            (total_distance start_anchor b))
;;
