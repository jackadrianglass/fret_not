open! Base
open Fret_not

let note degree octave alteration =
  Slot.Note
    (Degree_reference.create
       ~scale_degree:(Scale_degree.create ~degree ~alteration)
       ~octave)
;;

let as_tuples chunk =
  List.map chunk ~f:(function
    | Slot.Rest -> None
    | Slot.Note (dr : Degree_reference.t) ->
        Some
          ( Degree_reference.degree dr
          , Degree_reference.octave dr
          , Alteration.semitones (Degree_reference.alteration dr) ))
;;

let tuple = Alcotest.(option (triple int int int))

let reframing_over_the_tonic_step_is_a_no_op () =
  let chunk =
    [ note 1 0 Alteration.Natural; Slot.Rest; note 1 0 Alteration.Natural ]
  in
  Alcotest.(check (list tuple))
    "root degree 1 leaves degree-1 slots unchanged" (as_tuples chunk)
    (as_tuples (Chunk.reframe chunk ~root_degree:1))
;;

let reframing_over_the_fourth_shifts_the_tonic_to_degree_four () =
  let chunk = [ note 1 0 Alteration.Natural ] in
  Alcotest.(check (list tuple))
    "degree 1 over root degree 4 becomes degree 4"
    [ Some (4, 0, 0) ]
    (as_tuples (Chunk.reframe chunk ~root_degree:4))
;;

let reframing_carries_an_octave_when_the_degree_wraps_past_seven () =
  let chunk = [ note 5 0 Alteration.Natural ] in
  Alcotest.(check (list tuple))
    "degree 5 over root degree 5 wraps to degree 2, one octave higher"
    [ Some (2, 1, 0) ]
    (as_tuples (Chunk.reframe chunk ~root_degree:5))
;;

let reframing_preserves_alteration_and_existing_octave () =
  let chunk = [ note 2 1 Alteration.Flat ] in
  Alcotest.(check (list tuple))
    "alteration and any pre-existing octave carry through untouched"
    [ Some (5, 1, -1) ]
    (as_tuples (Chunk.reframe chunk ~root_degree:4))
;;

let rests_pass_through_unchanged () =
  let chunk = [ Slot.Rest; note 1 0 Alteration.Natural; Slot.Rest ] in
  Alcotest.(check (list tuple))
    "rests stay rests"
    [ None; Some (4, 0, 0); None ]
    (as_tuples (Chunk.reframe chunk ~root_degree:4))
;;

let tests =
  [ Alcotest.test_case "reframing over the tonic step is a no-op" `Quick
      reframing_over_the_tonic_step_is_a_no_op
  ; Alcotest.test_case
      "reframing over the fourth shifts the tonic to degree four" `Quick
      reframing_over_the_fourth_shifts_the_tonic_to_degree_four
  ; Alcotest.test_case
      "reframing carries an octave when the degree wraps past seven" `Quick
      reframing_carries_an_octave_when_the_degree_wraps_past_seven
  ; Alcotest.test_case "reframing preserves alteration and existing octave"
      `Quick reframing_preserves_alteration_and_existing_octave
  ; Alcotest.test_case "rests pass through unchanged" `Quick
      rests_pass_through_unchanged
  ]
;;
