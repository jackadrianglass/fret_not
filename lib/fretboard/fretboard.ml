open! Base

let mode_pitch_classes ~key ~mode =
  Mode.pitch_classes mode
    ~root:Spelled_pitch.(pitch_class (Key.mode_root key mode))
;;

let root_note ~key = Note.create ~spelled_pitch:(Key.tonic key) ~octave:0

let degree_note ~key ~mode (dr : Degree_reference.t) =
  Degree_reference.note ~root:(root_note ~key) ~mode dr
;;

let positions_of_semitone (instrument : Instrument.t) ~target_semitone =
  List.init (Instrument.string_count instrument) ~f:Fn.id
  |> List.filter_map ~f:(fun string_index ->
      let fret =
        target_semitone
        - Tuning.open_semitone (Instrument.tuning instrument) ~string_index
      in
      let position = { Fretboard_position.string_index; fret } in
      if Instrument.playable instrument position then Some position else None)
;;

let degree_positions ~instrument ~key ~mode (dr : Degree_reference.t) =
  positions_of_semitone instrument
    ~target_semitone:(Note.semitone (degree_note ~key ~mode dr))
;;

let degree_at ~instrument ~key ~mode (position : Fretboard_position.t) =
  let position_semitone = Instrument.semitone_at instrument position in
  let position_pitch_class = Pitch_class.of_int position_semitone in
  let matching_degree =
    List.find_mapi (mode_pitch_classes ~key ~mode) ~f:(fun i pitch_class ->
        if Pitch_class.equal pitch_class position_pitch_class then Some (i + 1)
        else None)
  in
  match matching_degree with
  | None -> None
  | Some degree ->
      let root = root_note ~key in
      let base =
        (Pitch_class.to_int position_pitch_class
        - Pitch_class.to_int (Note.pitch_class root)
        + 12)
        % 12
      in
      let position_interval = position_semitone - Note.semitone root in
      Some
        { Degree_reference.scale_degree = Scale_degree.natural ~degree
        ; octave = (position_interval - base) / 12
        }
;;

let degrees_in_window ~instrument ~key ~mode ~min_fret ~max_fret =
  List.init (Instrument.string_count instrument) ~f:Fn.id
  |> List.concat_map ~f:(fun string_index ->
      List.range min_fret (max_fret + 1)
      |> List.filter_map ~f:(fun fret ->
          let position = { Fretboard_position.string_index; fret } in
          if Instrument.playable instrument position then
            Option.map
              ~f:(fun degree -> (position, degree))
              (degree_at ~instrument ~key ~mode position)
          else None))
;;
