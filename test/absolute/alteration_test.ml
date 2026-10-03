open! Base
open Fret_not

let semitones_span_two_either_side_of_natural () =
  Alcotest.(check (list int))
    "semitones" [ -2; -1; 0; 1; 2 ]
    [ Alteration.semitones Double_flat
    ; Alteration.semitones Flat
    ; Alteration.semitones Natural
    ; Alteration.semitones Sharp
    ; Alteration.semitones Double_sharp
    ]
;;

let to_string_matches_notation () =
  Alcotest.(check (list string))
    "symbols"
    [ "bb"; "b"; ""; "#"; "##" ]
    [ Alteration.to_string Double_flat
    ; Alteration.to_string Flat
    ; Alteration.to_string Natural
    ; Alteration.to_string Sharp
    ; Alteration.to_string Double_sharp
    ]
;;

let compare_orders_by_semitones () =
  Alcotest.(check bool)
    "flats sort before natural, sharps after" true
    (Alteration.compare Flat Natural < 0 && Alteration.compare Sharp Natural > 0)
;;

let tests =
  [ Alcotest.test_case "semitones span two either side of natural" `Quick
      semitones_span_two_either_side_of_natural
  ; Alcotest.test_case "to_string matches notation" `Quick
      to_string_matches_notation
  ; Alcotest.test_case "compare orders by semitones" `Quick
      compare_orders_by_semitones
  ]
;;
