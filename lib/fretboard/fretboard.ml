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

let to_positions ~key ~mode ~tuning ~(anchor_position : Fretboard_position.t)
    (dr : Degree_reference.t) =
  let pitch_class = target_pitch_class ~key ~mode dr in
  let target =
    nearest_occurrence tuning ~anchor_position ~pitch_class + (12 * dr.octave)
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

(* `/` truncates toward zero, so a plain a/b would round the wrong way for
   negative octave bounds; these stay exact because a % b (Base's %, always
   same sign as b) makes the subtraction land on a multiple of b first. *)
let floor_div a b = (a - (a % b)) / b
let ceil_div a b = -(floor_div (-a) b)

let positions_in_window ~key ~mode ~tuning
    ~(anchor_position : Fretboard_position.t) ~min_fret ~max_fret =
  let last_string = Tuning.string_count tuning - 1 in
  let min_semitone =
    Tuning.relative_semitone tuning ~string_index:0 ~fret:min_fret
  in
  let max_semitone =
    Tuning.relative_semitone tuning ~string_index:last_string ~fret:max_fret
  in
  List.concat_map [ 1; 2; 3; 4; 5; 6; 7 ] ~f:(fun degree ->
      let pitch_class =
        target_pitch_class ~key ~mode
          { Degree_reference.degree; octave = 0; alteration = 0 }
      in
      let base = nearest_occurrence tuning ~anchor_position ~pitch_class in
      let octave_lo = floor_div (min_semitone - base) 12 in
      let octave_hi = ceil_div (max_semitone - base) 12 in
      List.concat_map
        (List.range octave_lo (octave_hi + 1))
        ~f:(fun octave ->
          to_positions ~key ~mode ~tuning ~anchor_position
            { Degree_reference.degree; octave; alteration = 0 }))
  |> List.filter ~f:(fun (p : Fretboard_position.t) ->
      p.fret >= min_fret && p.fret <= max_fret)
  |> List.sort ~compare:(fun (a : Fretboard_position.t) b ->
      match Int.compare a.string_index b.string_index with
      | 0 -> Int.compare a.fret b.fret
      | c -> c)
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
