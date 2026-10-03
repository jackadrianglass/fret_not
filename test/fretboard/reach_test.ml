open! Base
open Fret_not

let standard_instrument =
  Instrument.create_uniform ~tuning:Tuning.standard ~fret_count:22
;;

let position string_index fret : Fretboard_position.t = { string_index; fret }

let between_measures_fret_and_string_spans () =
  let reach = Reach.between (position 0 3) (position 2 6) in
  Alcotest.(check int) "fret span" 3 reach.fret_span;
  Alcotest.(check int) "string span" 2 reach.string_span
;;

let of_positions_bounding_boxes_the_shape () =
  let reach =
    Reach.of_positions [ position 0 3; position 2 6; position 1 0 ]
    |> Option.value_exn
  in
  Alcotest.(check int) "fret span 0 to 6" 6 reach.fret_span;
  Alcotest.(check int) "string span 0 to 2" 2 reach.string_span;
  Alcotest.(check bool)
    "no positions is None" true
    (Option.is_none (Reach.of_positions []))
;;

let semitone_span_accounts_for_cross_string_intervals () =
  (* Same 10-fret gap, but across different strings the sounding interval
     differs by the tuning's interval between the open strings. *)
  Alcotest.(check int)
    "same string is the fret delta" 10
    (Reach.semitone_span ~instrument:standard_instrument (position 0 0)
       (position 0 10));
  Alcotest.(check int)
    "low E fret 0 to D fret 5 is 15 semitones" 15
    (Reach.semitone_span ~instrument:standard_instrument (position 0 0)
       (position 2 5));
  Alcotest.(check int)
    "low E fret 8 to A fret 3 sounds the same note" 0
    (Reach.semitone_span ~instrument:standard_instrument (position 0 8)
       (position 1 3))
;;

let tests =
  [ Alcotest.test_case "between measures fret and string spans" `Quick
      between_measures_fret_and_string_spans
  ; Alcotest.test_case "of_positions bounding boxes the shape" `Quick
      of_positions_bounding_boxes_the_shape
  ; Alcotest.test_case "semitone_span accounts for cross-string intervals"
      `Quick semitone_span_accounts_for_cross_string_intervals
  ]
;;
