open! Base
open Fret_not

let c_major_state = Fretboard_view_state.initial
let config = Fretboard_view_config.default

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

let page_index_selects_the_page () =
  Alcotest.(check bool)
    "index 0 is the chunk editor" true
    (Fretboard_view_state.equal_page
       (Fretboard_view_state.page_of_index 0)
       Fretboard_view_state.Chunk_editing);
  Alcotest.(check bool)
    "any other index is the tab viewer" true
    (Fretboard_view_state.equal_page
       (Fretboard_view_state.page_of_index 1)
       Fretboard_view_state.Tab_viewing)
;;

let highlighted_positions_stay_in_key_and_on_the_instrument () =
  let positions =
    Fretboard_view_state.highlighted_positions c_major_state ~config
  in
  let drawn_frets = Instrument.max_fret config.instrument in
  Alcotest.(check bool) "non-empty" true (not (List.is_empty positions));
  Alcotest.(check bool)
    "within the instrument's frets" true
    (List.for_all positions ~f:(fun (p : Fretboard_position.t) ->
         p.fret >= 0 && p.fret <= drawn_frets));
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

let tests =
  [ Alcotest.test_case "tonics index like the pitch classes they spell" `Quick
      tonics_index_like_the_pitch_classes_they_spell
  ; Alcotest.test_case "key resolves the selected tonic spelling" `Quick
      key_resolves_the_selected_tonic_spelling
  ; Alcotest.test_case "page_index selects the page" `Quick
      page_index_selects_the_page
  ; Alcotest.test_case "highlighted positions stay in key and on the instrument"
      `Quick highlighted_positions_stay_in_key_and_on_the_instrument
  ; Alcotest.test_case "position_label_text shows degree number or note name"
      `Quick position_label_text_shows_degree_number_or_note_name
  ]
;;
