open! Base
open Fret_not

let pitch_class_wraps_past_b () =
  Alcotest.(check string)
    "B + 1 semitone" "C"
    (Pitch_class.to_string (Pitch_class.add (Pitch_class.of_int 11) 1))
;;

let pitch_class_wraps_below_c_with_negative_semitones () =
  Alcotest.(check string)
    "C - 1 semitone" "B"
    (Pitch_class.to_string (Pitch_class.add Pitch_class.c (-1)));
  Alcotest.(check string)
    "E - 9 semitones (the minor-key parent-major trick)" "G"
    (Pitch_class.to_string (Pitch_class.add Pitch_class.e (-9)))
;;

let tests =
  [ Alcotest.test_case "pitch class wraps" `Quick pitch_class_wraps_past_b
  ; Alcotest.test_case "pitch class wraps with negative semitones" `Quick
      pitch_class_wraps_below_c_with_negative_semitones
  ]
;;
