open! Base
open Fret_not

let c_major_state = Fretboard_view_state.initial
let config = Fretboard_view_config.default

let selected_position_follows_the_position_index () =
  Alcotest.(check bool)
    "None at position_index 0" true
    (Option.is_none
       (Fretboard_view_state.selected_position c_major_state ~config));
  let state = { c_major_state with position_index = 1 } in
  let expected =
    List.hd_exn
      (Fretboard_view_state.three_notes_per_string_positions state ~config)
  in
  Alcotest.(check bool)
    "position_index 1 selects the first three-notes-per-string shape" true
    (Option.equal
       (List.equal Fretboard_position.equal)
       (Some expected)
       (Fretboard_view_state.selected_position state ~config))
;;

let position_options_offer_all_plus_seven_shapes () =
  Alcotest.(check string)
    "All;1;..;7" "All;1;2;3;4;5;6;7"
    (Fretboard_view_state.position_options c_major_state)
;;

let tonics_index_like_the_pitch_classes_they_spell () =
  Alcotest.(check int)
    "one tonic per dropdown entry" 12
    (List.length Fretboard_view_state.tonics);
  Alcotest.(check (list string))
    "first three and last entries" [ "C"; "C#"; "D"; "B" ]
    (List.map
       [ List.nth_exn Fretboard_view_state.tonics 0
       ; List.nth_exn Fretboard_view_state.tonics 1
       ; List.nth_exn Fretboard_view_state.tonics 2
       ; List.nth_exn Fretboard_view_state.tonics 11
       ]
       ~f:Spelled_pitch.to_string)
;;

let key_resolves_the_selected_tonic_spelling () =
  let f_major =
    Fretboard_view_state.key { c_major_state with tonic_index = 5 }
  in
  Alcotest.(check bool)
    "tonic_index 5 is F" true
    (Pitch_class.equal Pitch_class.f (Key.tonic_pitch_class f_major))
;;

let highlighted_positions_stay_in_key_and_on_the_instrument () =
  let positions =
    Fretboard_view_state.highlighted_positions c_major_state ~config
  in
  Alcotest.(check bool) "non-empty" true (not (List.is_empty positions));
  Alcotest.(check bool)
    "within the drawn fret window" true
    (List.for_all positions ~f:(fun (p : Fretboard_position.t) ->
         p.fret >= 0 && p.fret <= config.fret_count));
  Alcotest.(check bool)
    "playable on the instrument" true
    (List.for_all positions ~f:(fun position ->
         Instrument.playable config.instrument position));
  let chromatic : Fretboard_position.t = { string_index = 0; fret = 6 } in
  Alcotest.(check bool)
    "off-key positions excluded" true
    (not (List.mem positions chromatic ~equal:Fretboard_position.equal))
;;

let position_label_text_shows_degree_number_or_note_name () =
  let root = Spelled_pitch.natural Letter.C in
  Alcotest.(check string)
    "degree number" "1"
    (Fretboard_view_state.position_label_text ~root ~mode:Mode.Ionian
       ~label_mode:Fretboard_view_state.Degree_number
       (Scale_degree.natural ~degree:1));
  Alcotest.(check string)
    "note name" "C"
    (Fretboard_view_state.position_label_text ~root ~mode:Mode.Ionian
       ~label_mode:Fretboard_view_state.Note_name
       (Scale_degree.natural ~degree:1));
  Alcotest.(check string)
    "third of A major is spelled C#" "C#"
    (Fretboard_view_state.position_label_text
       ~root:(Spelled_pitch.natural Letter.A)
       ~mode:Mode.Ionian ~label_mode:Fretboard_view_state.Note_name
       (Scale_degree.natural ~degree:3))
;;

let scale_is_diatonic_at_every_index () =
  List.iter [ 0; 1; 2 ] ~f:(fun index ->
      Alcotest.(check bool)
        (Printf.sprintf "scale_of_index %d" index)
        true
        (Poly.equal
           (Fretboard_view_state.scale_of_index index)
           Fretboard_view_state.Diatonic))
;;

let tests =
  [ Alcotest.test_case "selected_position follows the position index" `Quick
      selected_position_follows_the_position_index
  ; Alcotest.test_case "position_options offer All plus seven shapes" `Quick
      position_options_offer_all_plus_seven_shapes
  ; Alcotest.test_case "tonics index like the pitch classes they spell" `Quick
      tonics_index_like_the_pitch_classes_they_spell
  ; Alcotest.test_case "key resolves the selected tonic spelling" `Quick
      key_resolves_the_selected_tonic_spelling
  ; Alcotest.test_case "highlighted positions stay in key and on the instrument"
      `Quick highlighted_positions_stay_in_key_and_on_the_instrument
  ; Alcotest.test_case "position_label_text shows degree number or note name"
      `Quick position_label_text_shows_degree_number_or_note_name
  ; Alcotest.test_case "scale is Diatonic at every index" `Quick
      scale_is_diatonic_at_every_index
  ]
;;
