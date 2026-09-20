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
  ]
;;
