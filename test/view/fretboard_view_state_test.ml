open! Base
open Fret_not

let c_major_state = Fretboard_view_state.initial

let selected_position_is_none_at_position_index_zero () =
  Alcotest.(check bool)
    "None when position_index is 0" true
    (Option.is_none
       (Fretboard_view_state.selected_position c_major_state
          ~config:Fretboard_view_config.default))
;;

let selected_position_matches_three_notes_per_string_for_diatonic () =
  let state = { c_major_state with position_index = 1 } in
  let expected =
    List.hd_exn
      (Fretboard_view_state.three_notes_per_string_positions state
         ~config:Fretboard_view_config.default)
  in
  Alcotest.(check bool)
    "Diatonic position 1 matches three_notes_per_string_positions" true
    (Option.equal
       (List.equal Fretboard_position.equal)
       (Some expected)
       (Fretboard_view_state.selected_position state
          ~config:Fretboard_view_config.default))
;;

let selected_position_matches_two_notes_per_string_for_pentatonic () =
  let state = { c_major_state with scale_index = 1; position_index = 1 } in
  let expected =
    List.hd_exn
      (Fretboard_view_state.two_notes_per_string_positions state
         ~config:Fretboard_view_config.default)
  in
  Alcotest.(check bool)
    "Pentatonic position 1 matches two_notes_per_string_positions" true
    (Option.equal
       (List.equal Fretboard_position.equal)
       (Some expected)
       (Fretboard_view_state.selected_position state
          ~config:Fretboard_view_config.default))
;;

let selected_position_matches_one_note_per_string_for_arpeggio () =
  let state = { c_major_state with scale_index = 2; position_index = 1 } in
  let expected =
    List.hd_exn
      (Fretboard_view_state.one_note_per_string_positions state
         ~config:Fretboard_view_config.default)
  in
  Alcotest.(check bool)
    "Arpeggio position 1 matches one_note_per_string_positions" true
    (Option.equal
       (List.equal Fretboard_position.equal)
       (Some expected)
       (Fretboard_view_state.selected_position state
          ~config:Fretboard_view_config.default))
;;

let position_options_diatonic_has_all_plus_seven_modes () =
  Alcotest.(check int)
    "All + 7 modes" 8
    (String.split (Fretboard_view_state.position_options c_major_state) ~on:';'
    |> List.length)
;;

let position_options_pentatonic_has_all_plus_five () =
  let state = { c_major_state with scale_index = 1 } in
  Alcotest.(check string)
    "All;1;2;3;4;5" "All;1;2;3;4;5"
    (Fretboard_view_state.position_options state)
;;

let position_options_arpeggio_has_all_plus_three_inversions () =
  let state = { c_major_state with scale_index = 2 } in
  Alcotest.(check string)
    "All;Root;1st Inv;2nd Inv" "All;Root;1st Inv;2nd Inv"
    (Fretboard_view_state.position_options state)
;;

let position_label_text_shows_degree_number_or_note_name () =
  let scale_degree : Scale_degree.t = Scale_degree.natural ~degree:1 in
  Alcotest.(check string)
    "degree number" "1"
    (Fretboard_view_state.position_label_text ~root:Pitch_class.c
       ~mode:Mode.Ionian ~label_mode:Fretboard_view_state.Degree_number
       scale_degree);
  Alcotest.(check string)
    "note name" "C"
    (Fretboard_view_state.position_label_text ~root:Pitch_class.c
       ~mode:Mode.Ionian ~label_mode:Fretboard_view_state.Note_name scale_degree)
;;

let index_converters_round_trip () =
  Alcotest.(check bool)
    "0 -> Major" true
    (Poly.equal (Fretboard_view_state.quality_of_index 0) Key.Major);
  Alcotest.(check bool)
    "1 -> Minor" true
    (Poly.equal (Fretboard_view_state.quality_of_index 1) Key.Minor);
  Alcotest.(check bool)
    "0 -> Diatonic" true
    (Poly.equal
       (Fretboard_view_state.scale_of_index 0)
       Fretboard_view_state.Diatonic);
  Alcotest.(check bool)
    "1 -> Pentatonic" true
    (Poly.equal
       (Fretboard_view_state.scale_of_index 1)
       Fretboard_view_state.Pentatonic);
  Alcotest.(check bool)
    "2 -> Arpeggio" true
    (Poly.equal
       (Fretboard_view_state.scale_of_index 2)
       Fretboard_view_state.Arpeggio)
;;

let tests =
  [ Alcotest.test_case "selected_position is None at position_index 0" `Quick
      selected_position_is_none_at_position_index_zero
  ; Alcotest.test_case "selected_position matches 3nps for Diatonic" `Quick
      selected_position_matches_three_notes_per_string_for_diatonic
  ; Alcotest.test_case "selected_position matches 2nps for Pentatonic" `Quick
      selected_position_matches_two_notes_per_string_for_pentatonic
  ; Alcotest.test_case "selected_position matches 1nps for Arpeggio" `Quick
      selected_position_matches_one_note_per_string_for_arpeggio
  ; Alcotest.test_case "position_options: Diatonic has All + 7 modes" `Quick
      position_options_diatonic_has_all_plus_seven_modes
  ; Alcotest.test_case "position_options: Pentatonic has All + 5" `Quick
      position_options_pentatonic_has_all_plus_five
  ; Alcotest.test_case "position_options: Arpeggio has All + 3 inversions"
      `Quick position_options_arpeggio_has_all_plus_three_inversions
  ; Alcotest.test_case "position_label_text shows degree number or note name"
      `Quick position_label_text_shows_degree_number_or_note_name
  ; Alcotest.test_case "index converters round trip" `Quick
      index_converters_round_trip
  ]
;;
