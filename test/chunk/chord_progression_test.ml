open! Base
open Fret_not

let note degree = Slot.Note (Degree_reference.natural ~degree ~octave:0)

let as_degrees chunk =
  List.map chunk ~f:(function
    | Slot.Rest -> None
    | Slot.Note (dr : Degree_reference.t) -> Some (Degree_reference.degree dr))
;;

let applying_a_i_iv_v_progression_reinterprets_a_bare_tonic_chunk () =
  let chunk = [ note 1 ] in
  let progression : Chord_progression.t = [ 1; 4; 5 ] in
  let reprojected = Chord_progression.apply_to_chunk progression chunk in
  Alcotest.(check (list (list (option int))))
    "I-IV-V reprojection of a bare tonic chunk"
    [ [ Some 1 ]; [ Some 4 ]; [ Some 5 ] ]
    (List.map reprojected ~f:as_degrees)
;;

let apply_to_chunk_produces_one_chunk_per_step () =
  let progression : Chord_progression.t = [ 1; 4; 5 ] in
  Alcotest.(check int)
    "one reprojected chunk per progression step" 3
    (List.length (Chord_progression.apply_to_chunk progression [ note 1 ]))
;;

let tests =
  [ Alcotest.test_case
      "applying a I-IV-V progression reinterprets a bare tonic chunk" `Quick
      applying_a_i_iv_v_progression_reinterprets_a_bare_tonic_chunk
  ; Alcotest.test_case "apply_to_chunk produces one chunk per step" `Quick
      apply_to_chunk_produces_one_chunk_per_step
  ]
;;
