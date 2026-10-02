open! Base

let mode_degrees ~key ~mode = Mode.degrees mode ~root:(Key.mode_root key mode)

let target_pitch_class ~key ~mode (dr : Degree_reference.t) =
  let base_degree : Scale_degree.t =
    List.nth_exn (mode_degrees ~key ~mode) (dr.degree - 1)
  in
  Pitch_class.add base_degree.pitch_class dr.alteration
;;

let relative_semitone_of_position tuning (position : Fretboard_position.t) =
  Tuning.relative_semitone tuning ~string_index:position.string_index
    ~fret:position.fret
;;

let pitch_class_of_position tuning (position : Fretboard_position.t) =
  Tuning.pitch_class_at tuning ~string_index:position.string_index
    ~fret:position.fret
;;

let nearest_occurrence tuning ~(anchor_position : Fretboard_position.t)
    ~pitch_class =
  let anchor = relative_semitone_of_position tuning anchor_position in
  let up_step =
    (Pitch_class.to_int pitch_class
    - Pitch_class.to_int (pitch_class_of_position tuning anchor_position)
    + 12)
    % 12
  in
  if up_step <= 6 then anchor + up_step else anchor + up_step - 12
;;

let positions_for_pitch_class tuning ~(anchor_position : Fretboard_position.t)
    ~pitch_class ~octave =
  let target =
    nearest_occurrence tuning ~anchor_position ~pitch_class + (12 * octave)
  in
  List.init (Tuning.string_count tuning) ~f:Fn.id
  |> List.filter_map ~f:(fun string_index ->
      let open_offset = Tuning.relative_semitone tuning ~string_index ~fret:0 in
      let fret = target - open_offset in
      if fret >= 0 then Some { Fretboard_position.string_index; fret } else None)
  |> List.sort
       ~compare:(fun (a : Fretboard_position.t) (b : Fretboard_position.t) ->
         Int.compare
           (Int.abs (a.fret - anchor_position.fret))
           (Int.abs (b.fret - anchor_position.fret)))
;;

let to_positions ~key ~mode ~tuning ~(anchor_position : Fretboard_position.t)
    (dr : Degree_reference.t) =
  let pitch_class = target_pitch_class ~key ~mode dr in
  positions_for_pitch_class tuning ~anchor_position ~pitch_class
    ~octave:dr.octave
;;

let floor_div a b = (a - (a % b)) / b
let ceil_div a b = -(floor_div (-a) b)

let positions_in_window_for_degrees tuning
    ~(anchor_position : Fretboard_position.t) ~min_fret ~max_fret
    (degrees : Scale_degree.t list) =
  let last_string = Tuning.string_count tuning - 1 in
  let min_semitone =
    Tuning.relative_semitone tuning ~string_index:0 ~fret:min_fret
  in
  let max_semitone =
    Tuning.relative_semitone tuning ~string_index:last_string ~fret:max_fret
  in
  List.concat_map degrees ~f:(fun (d : Scale_degree.t) ->
      let base =
        nearest_occurrence tuning ~anchor_position ~pitch_class:d.pitch_class
      in
      let octave_lo = floor_div (min_semitone - base) 12 in
      let octave_hi = ceil_div (max_semitone - base) 12 in
      List.concat_map
        (List.range octave_lo (octave_hi + 1))
        ~f:(fun octave ->
          positions_for_pitch_class tuning ~anchor_position
            ~pitch_class:d.pitch_class ~octave))
  |> List.filter ~f:(fun (p : Fretboard_position.t) ->
      p.fret >= min_fret && p.fret <= max_fret)
  |> List.sort ~compare:(fun (a : Fretboard_position.t) b ->
      match Int.compare a.string_index b.string_index with
      | 0 -> Int.compare a.fret b.fret
      | c -> c)
;;

let positions_in_window ~key ~mode ~tuning ~anchor_position ~min_fret ~max_fret
    =
  positions_in_window_for_degrees tuning ~anchor_position ~min_fret ~max_fret
    (mode_degrees ~key ~mode)
;;

let pentatonic_degrees ~key =
  match Key.quality key with
  | Key.Major -> Pentatonic.major ~root:(Key.tonic key)
  | Key.Minor -> Pentatonic.minor ~root:(Key.tonic key)
;;

let pentatonic_positions_in_window ~key ~tuning ~anchor_position ~min_fret
    ~max_fret =
  positions_in_window_for_degrees tuning ~anchor_position ~min_fret ~max_fret
    (pentatonic_degrees ~key)
;;

let rec ascending_diatonic_frets tuning ~diatonic_pitch_classes ~string_index
    ~min_relative_semitone ~count ~fret =
  if count = 0 then []
  else
    let semitone = Tuning.relative_semitone tuning ~string_index ~fret in
    let pitch_class = Tuning.pitch_class_at tuning ~string_index ~fret in
    if
      semitone >= min_relative_semitone
      && List.mem diatonic_pitch_classes pitch_class ~equal:Pitch_class.equal
    then
      { Fretboard_position.string_index; fret }
      :: ascending_diatonic_frets tuning ~diatonic_pitch_classes ~string_index
           ~min_relative_semitone ~count:(count - 1) ~fret:(fret + 1)
    else
      ascending_diatonic_frets tuning ~diatonic_pitch_classes ~string_index
        ~min_relative_semitone ~count ~fret:(fret + 1)
;;

let ascending_diatonic_positions tuning ~diatonic_pitch_classes ~string_index
    ~min_relative_semitone ~count =
  let open_offset = Tuning.relative_semitone tuning ~string_index ~fret:0 in
  let start_fret = Int.max 0 (min_relative_semitone - open_offset) in
  ascending_diatonic_frets tuning ~diatonic_pitch_classes ~string_index
    ~min_relative_semitone ~count ~fret:start_fret
;;

let one_notes_per_string_position tuning ~diatonic_pitch_classes
    ~notes_per_string ~string_count ~low_string_min_semitone =
  let rec across_strings string_index min_semitone =
    if string_index = string_count then []
    else
      let notes =
        ascending_diatonic_positions tuning ~diatonic_pitch_classes
          ~string_index ~min_relative_semitone:min_semitone
          ~count:notes_per_string
      in
      let next_min_semitone =
        relative_semitone_of_position tuning (List.last_exn notes) + 1
      in
      notes @ across_strings (string_index + 1) next_min_semitone
  in
  across_strings 0 low_string_min_semitone
;;

let dropped_to_lowest_playable_octave position =
  let min_fret =
    List.map position ~f:(fun (p : Fretboard_position.t) -> p.fret)
    |> List.min_elt ~compare:Int.compare
    |> Option.value_exn
  in
  let octave_drop = min_fret / 12 * 12 in
  List.map position ~f:(fun (p : Fretboard_position.t) ->
      { p with fret = p.fret - octave_drop })
;;

let relative_semitone_of_pitch_class tuning ~pitch_class =
  let open_string_pitch_class =
    Tuning.pitch_class_at tuning ~string_index:0 ~fret:0
  in
  (Pitch_class.to_int pitch_class
  - Pitch_class.to_int open_string_pitch_class
  + 12)
  % 12
;;

let one_step_past_low_string_start tuning ~diatonic_pitch_classes
    ~low_string_start_semitone =
  ascending_diatonic_positions tuning ~diatonic_pitch_classes ~string_index:0
    ~min_relative_semitone:(low_string_start_semitone + 1)
    ~count:1
  |> List.hd_exn
  |> relative_semitone_of_position tuning
;;

let notes_per_string_positions ~notes_per_string ~diatonic_pitch_classes ~tuning
    ~root_relative_semitone =
  let string_count = Tuning.string_count tuning in
  let position_count = List.length diatonic_pitch_classes in
  let rec positions remaining low_string_min_semitone =
    if remaining = 0 then []
    else
      let this_position =
        one_notes_per_string_position tuning ~diatonic_pitch_classes
          ~notes_per_string ~string_count ~low_string_min_semitone
      in
      let low_string_start_semitone =
        relative_semitone_of_position tuning (List.hd_exn this_position)
      in
      let next_low_string_min_semitone =
        one_step_past_low_string_start tuning ~diatonic_pitch_classes
          ~low_string_start_semitone
      in
      dropped_to_lowest_playable_octave this_position
      :: positions (remaining - 1) next_low_string_min_semitone
  in
  positions position_count root_relative_semitone
;;

let three_notes_per_string_positions ~key ~mode ~tuning =
  let diatonic_pitch_classes =
    List.map (mode_degrees ~key ~mode) ~f:(fun (d : Scale_degree.t) ->
        d.pitch_class)
  in
  let root_relative_semitone =
    relative_semitone_of_pitch_class tuning
      ~pitch_class:(Key.mode_root key mode)
  in
  notes_per_string_positions ~notes_per_string:3 ~diatonic_pitch_classes ~tuning
    ~root_relative_semitone
;;

let two_notes_per_string_positions ~key ~tuning =
  let diatonic_pitch_classes =
    List.map (pentatonic_degrees ~key) ~f:(fun (d : Scale_degree.t) ->
        d.pitch_class)
  in
  let root_relative_semitone =
    relative_semitone_of_pitch_class tuning ~pitch_class:(Key.tonic key)
  in
  notes_per_string_positions ~notes_per_string:2 ~diatonic_pitch_classes ~tuning
    ~root_relative_semitone
;;

let arpeggio_degrees ~key =
  match Key.quality key with
  | Key.Major -> Arpeggio.major ~root:(Key.tonic key)
  | Key.Minor -> Arpeggio.minor ~root:(Key.tonic key)
;;

let arpeggio_positions_in_window ~key ~tuning ~anchor_position ~min_fret
    ~max_fret =
  positions_in_window_for_degrees tuning ~anchor_position ~min_fret ~max_fret
    (arpeggio_degrees ~key)
;;

let with_closing_note tuning ~diatonic_pitch_classes position =
  let last = List.last_exn position in
  let closing_note =
    ascending_diatonic_positions tuning ~diatonic_pitch_classes
      ~string_index:last.Fretboard_position.string_index
      ~min_relative_semitone:(relative_semitone_of_position tuning last + 1)
      ~count:1
    |> List.hd_exn
  in
  position @ [ closing_note ]
;;

let one_note_per_string_positions ~key ~tuning =
  let diatonic_pitch_classes =
    List.map (arpeggio_degrees ~key) ~f:(fun (d : Scale_degree.t) ->
        d.pitch_class)
  in
  let root_relative_semitone =
    relative_semitone_of_pitch_class tuning ~pitch_class:(Key.tonic key)
  in
  notes_per_string_positions ~notes_per_string:1 ~diatonic_pitch_classes ~tuning
    ~root_relative_semitone
  |> List.map ~f:(with_closing_note tuning ~diatonic_pitch_classes)
;;

let of_position ~key ~mode ~tuning ~(anchor_position : Fretboard_position.t)
    (position : Fretboard_position.t) =
  let pos_pitch_class = pitch_class_of_position tuning position in
  let matching : Scale_degree.t =
    List.find_exn (mode_degrees ~key ~mode) ~f:(fun (d : Scale_degree.t) ->
        Pitch_class.equal d.pitch_class pos_pitch_class)
  in
  let anchor_occurrence =
    nearest_occurrence tuning ~anchor_position ~pitch_class:pos_pitch_class
  in
  let pos_semitone = relative_semitone_of_position tuning position in
  { Degree_reference.degree = matching.degree
  ; octave = (pos_semitone - anchor_occurrence) / 12
  ; alteration = 0
  }
;;
