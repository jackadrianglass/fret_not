open! Base
open Fret_not

let pitch_class_wraps_past_b () =
  Alcotest.(check string)
    "B + 1 semitone" "C"
    (Pitch_class.to_string (Pitch_class.add (Pitch_class.of_int 11) 1))
;;

let tests =
  [ Alcotest.test_case "pitch class wraps" `Quick pitch_class_wraps_past_b ]
;;
