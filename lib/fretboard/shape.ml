open! Base

let scale_notes ~(degrees : Scale_degree.t list) ~start ~count =
  let cycle = List.length degrees in
  if cycle = 0 then []
  else
    List.init count ~f:(fun i ->
        let index = start + i in
        Degree_reference.create
          ~scale_degree:(List.nth_exn degrees (index % cycle))
          ~octave:(index / cycle))
;;

let start_semitone ~instrument ~key (anchor : Fretboard_position.t) =
  let anchor_semitone = Instrument.semitone_at instrument anchor in
  anchor_semitone
  + (Pitch_class.to_int (Key.tonic_pitch_class key)
    - Pitch_class.to_int (Pitch_class.of_int anchor_semitone)
    + 12)
    % 12
;;

let positions ~instrument ~key ~mode ~degrees ~notes_per_string
    ~(anchor : Fretboard_position.t) =
  let string_count = Instrument.string_count instrument in
  let shape_count = List.length degrees in
  let note_count = string_count * notes_per_string in
  let drop_to_lowest_playable_octave frets =
    let min_fret =
      List.min_elt frets ~compare:Int.compare |> Option.value_exn
    in
    let octave_drop = Int.max 0 min_fret / 12 * 12 in
    List.map frets ~f:(fun fret -> fret - octave_drop)
  in
  List.init shape_count ~f:(fun shape_start ->
      let notes = scale_notes ~degrees ~start:shape_start ~count:note_count in
      let frets =
        List.mapi notes ~f:(fun note_index (dr : Degree_reference.t) ->
            let string_index = note_index / notes_per_string in
            let semitone =
              start_semitone ~instrument ~key anchor
              + Degree_reference.interval
                  ~root:(Key.tonic_pitch_class key)
                  ~mode dr
            in
            semitone
            - Tuning.open_semitone (Instrument.tuning instrument) ~string_index)
      in
      let dropped = drop_to_lowest_playable_octave frets in
      List.mapi dropped ~f:(fun note_index fret ->
          { Fretboard_position.string_index = note_index / notes_per_string
          ; fret
          }))
  |> List.filter ~f:(fun shape ->
      List.for_all shape ~f:(fun position ->
          Instrument.playable instrument position))
;;
