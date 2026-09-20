open! Base
open Fret_not

let anchor_open_low_e : Fretboard_position.t = { string_index = 0; fret = 0 }
let c_major = Key.create ~tonic:(Pitch_class.of_int 0) ~quality:Major

let degree_one_octave_zero_is_unreachable_near_open_low_e () =
  let positions =
    Fretboard.to_positions ~key:c_major ~mode:Mode.Ionian
      ~tuning:Tuning.standard ~anchor_position:anchor_open_low_e
      { Degree_reference.degree = 1; octave = 0; alteration = 0 }
  in
  Alcotest.(check int)
    "no reachable C at octave 0 near open low E" 0 (List.length positions)
;;

let degree_one_octave_one_is_reachable_on_two_strings_nearest_first () =
  let positions =
    Fretboard.to_positions ~key:c_major ~mode:Mode.Ionian
      ~tuning:Tuning.standard ~anchor_position:anchor_open_low_e
      { Degree_reference.degree = 1; octave = 1; alteration = 0 }
  in
  Alcotest.(check (list (pair int int)))
    "A string fret 3 (closer to the anchor) before low E fret 8"
    [ (1, 3); (0, 8) ]
    (List.map positions ~f:(fun (p : Fretboard_position.t) ->
         (p.string_index, p.fret)))
;;

let degree_reference_round_trips_through_the_same_anchor () =
  let dr : Degree_reference.t = { degree = 1; octave = 1; alteration = 0 } in
  let position =
    List.hd_exn
      (Fretboard.to_positions ~key:c_major ~mode:Mode.Ionian
         ~tuning:Tuning.standard ~anchor_position:anchor_open_low_e dr)
  in
  Alcotest.(check bool)
    "round trip returns the original degree reference" true
    (Degree_reference.equal dr
       (Fretboard.of_position ~key:c_major ~mode:Mode.Ionian
          ~tuning:Tuning.standard ~anchor_position:anchor_open_low_e position))
;;

let of_position_raises_on_a_chromatic_pitch () =
  let chromatic_position : Fretboard_position.t =
    { string_index = 0; fret = 6 }
  in
  Alcotest.(check bool)
    "chromatic position raises" true
    (Exn.does_raise (fun () ->
         Fretboard.of_position ~key:c_major ~mode:Mode.Ionian
           ~tuning:Tuning.standard ~anchor_position:anchor_open_low_e
           chromatic_position))
;;

let full_c_major_scale_positions () =
  List.concat_map [ -1; 0; 1; 2; 3; 4 ] ~f:(fun octave ->
      List.concat_map [ 1; 2; 3; 4; 5; 6; 7 ] ~f:(fun degree ->
          Fretboard.to_positions ~key:c_major ~mode:Mode.Ionian
            ~tuning:Tuning.standard ~anchor_position:anchor_open_low_e
            { Degree_reference.degree; octave; alteration = 0 }))
  |> List.filter ~f:(fun (p : Fretboard_position.t) -> p.fret <= 12)
;;

let every_string_shows_all_seven_degrees_within_twelve_frets () =
  let positions = full_c_major_scale_positions () in
  let frets_on string_index =
    positions
    |> List.filter_map ~f:(fun (p : Fretboard_position.t) ->
        if p.string_index = string_index then Some p.fret else None)
    |> List.dedup_and_sort ~compare:Int.compare
  in
  Alcotest.(check (list int)) "low E" [ 0; 1; 3; 5; 7; 8; 10; 12 ] (frets_on 0);
  Alcotest.(check (list int)) "A" [ 0; 2; 3; 5; 7; 8; 10; 12 ] (frets_on 1);
  Alcotest.(check (list int)) "D" [ 0; 2; 3; 5; 7; 9; 10; 12 ] (frets_on 2);
  Alcotest.(check (list int)) "G" [ 0; 2; 4; 5; 7; 9; 10; 12 ] (frets_on 3);
  Alcotest.(check (list int)) "B" [ 0; 1; 3; 5; 6; 8; 10; 12 ] (frets_on 4);
  Alcotest.(check (list int)) "high E" [ 0; 1; 3; 5; 7; 8; 10; 12 ] (frets_on 5)
;;

let sorted_by_string_then_fret positions =
  List.sort positions ~compare:(fun (a : Fretboard_position.t) b ->
      match Int.compare a.string_index b.string_index with
      | 0 -> Int.compare a.fret b.fret
      | c -> c)
;;

let positions_in_window_matches_the_guessed_range_search () =
  let via_window =
    Fretboard.positions_in_window ~key:c_major ~mode:Mode.Ionian
      ~tuning:Tuning.standard ~anchor_position:anchor_open_low_e ~min_fret:0
      ~max_fret:12
  in
  Alcotest.(check (list (pair int int)))
    "positions_in_window matches the reference guessed-range search"
    (sorted_by_string_then_fret (full_c_major_scale_positions ())
    |> List.map ~f:(fun (p : Fretboard_position.t) -> (p.string_index, p.fret))
    )
    (sorted_by_string_then_fret via_window
    |> List.map ~f:(fun (p : Fretboard_position.t) -> (p.string_index, p.fret))
    )
;;

let three_notes_per_string_positions_are_seven_of_eighteen () =
  let positions =
    Fretboard.three_notes_per_string_positions ~key:c_major ~mode:Mode.Ionian
      ~tuning:Tuning.standard
  in
  Alcotest.(check int) "seven positions" 7 (List.length positions);
  List.iter positions ~f:(fun position ->
      Alcotest.(check int) "18 notes (6 strings x 3)" 18 (List.length position))
;;

let position_one_matches_the_known_c_major_ionian_shape () =
  let positions =
    Fretboard.three_notes_per_string_positions ~key:c_major ~mode:Mode.Ionian
      ~tuning:Tuning.standard
  in
  let position_one = List.hd_exn positions in
  Alcotest.(check (list (pair int int)))
    "C major Ionian position 1"
    [ (0, 8)
    ; (0, 10)
    ; (0, 12)
    ; (1, 8)
    ; (1, 10)
    ; (1, 12)
    ; (2, 9)
    ; (2, 10)
    ; (2, 12)
    ; (3, 9)
    ; (3, 10)
    ; (3, 12)
    ; (4, 10)
    ; (4, 12)
    ; (4, 13)
    ; (5, 10)
    ; (5, 12)
    ; (5, 13)
    ]
    (List.map position_one ~f:(fun (p : Fretboard_position.t) ->
         (p.string_index, p.fret)))
;;

let position_three_is_dropped_to_its_lowest_playable_octave () =
  let positions =
    Fretboard.three_notes_per_string_positions ~key:c_major ~mode:Mode.Ionian
      ~tuning:Tuning.standard
  in
  let position_three = List.nth_exn positions 2 in
  Alcotest.(check (list (pair int int)))
    "C major Phrygian position 3, dropped an octave to stay on the neck"
    [ (0, 0)
    ; (0, 1)
    ; (0, 3)
    ; (1, 0)
    ; (1, 2)
    ; (1, 3)
    ; (2, 0)
    ; (2, 2)
    ; (2, 3)
    ; (3, 0)
    ; (3, 2)
    ; (3, 4)
    ; (4, 1)
    ; (4, 3)
    ; (4, 5)
    ; (5, 1)
    ; (5, 3)
    ; (5, 5)
    ]
    (List.map position_three ~f:(fun (p : Fretboard_position.t) ->
         (p.string_index, p.fret)))
;;

let every_position_stays_within_a_playable_octave () =
  let positions =
    Fretboard.three_notes_per_string_positions ~key:c_major ~mode:Mode.Ionian
      ~tuning:Tuning.standard
  in
  List.iter positions ~f:(fun position ->
      let frets =
        List.map position ~f:(fun (p : Fretboard_position.t) -> p.fret)
      in
      let min_fret =
        List.min_elt frets ~compare:Int.compare |> Option.value_exn
      in
      Alcotest.(check bool)
        "lowest fret in each position is within the first octave" true
        (min_fret >= 0 && min_fret < 12))
;;

let positions_land_on_modes_in_sequential_order () =
  let positions =
    Fretboard.three_notes_per_string_positions ~key:c_major ~mode:Mode.Ionian
      ~tuning:Tuning.standard
  in
  let position_mode_name (position : Fretboard_position.t list) =
    let first = List.hd_exn position in
    let pitch_class =
      Tuning.pitch_class_at Tuning.standard ~string_index:first.string_index
        ~fret:first.fret
    in
    Mode.name (Option.value_exn (Key.mode_with_root c_major pitch_class))
  in
  Alcotest.(check (list string))
    "positions land on modes in Ionian..Locrian order"
    (List.map Mode.all ~f:Mode.name)
    (List.map positions ~f:position_mode_name)
;;

let tests =
  [ Alcotest.test_case "octave 0 unreachable near open low E" `Quick
      degree_one_octave_zero_is_unreachable_near_open_low_e
  ; Alcotest.test_case "octave 1 reachable on two strings, nearest first" `Quick
      degree_one_octave_one_is_reachable_on_two_strings_nearest_first
  ; Alcotest.test_case "round trip through the same anchor" `Quick
      degree_reference_round_trips_through_the_same_anchor
  ; Alcotest.test_case "of_position raises on a chromatic pitch" `Quick
      of_position_raises_on_a_chromatic_pitch
  ; Alcotest.test_case "every string shows all 7 degrees within 12 frets" `Quick
      every_string_shows_all_seven_degrees_within_twelve_frets
  ; Alcotest.test_case "positions_in_window matches the guessed-range search"
      `Quick positions_in_window_matches_the_guessed_range_search
  ; Alcotest.test_case "three_notes_per_string_positions are seven of eighteen"
      `Quick three_notes_per_string_positions_are_seven_of_eighteen
  ; Alcotest.test_case "position 1 matches the known C major Ionian shape"
      `Quick position_one_matches_the_known_c_major_ionian_shape
  ; Alcotest.test_case "positions land on modes in sequential order" `Quick
      positions_land_on_modes_in_sequential_order
  ; Alcotest.test_case "position 3 is dropped to its lowest playable octave"
      `Quick position_three_is_dropped_to_its_lowest_playable_octave
  ; Alcotest.test_case "every position stays within a playable octave" `Quick
      every_position_stays_within_a_playable_octave
  ]
;;
