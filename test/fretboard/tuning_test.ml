open! Base
open Fret_not

let open_strings tuning =
  List.init (Tuning.string_count tuning) ~f:(fun string_index ->
      Pitch_class.to_string (Tuning.pitch_class_at tuning ~string_index ~fret:0))
;;

let standard_tuning_open_strings_are_e_a_d_g_b_e () =
  Alcotest.(check (list string))
    "EADGBE"
    [ "E"; "A"; "D"; "G"; "B"; "E" ]
    (open_strings Tuning.standard)
;;

let drop_d_only_lowers_the_low_string () =
  Alcotest.(check (list string))
    "DADGBE"
    [ "D"; "A"; "D"; "G"; "B"; "E" ]
    (open_strings Tuning.drop_d)
;;

let seven_string_tuning_has_seven_strings () =
  Alcotest.(check int)
    "string count" 7
    (Tuning.string_count Tuning.standard_seven_string)
;;

let fret_twelve_returns_to_the_open_pitch_class () =
  Alcotest.(check string)
    "12th fret" "E"
    (Pitch_class.to_string
       (Tuning.pitch_class_at Tuning.standard ~string_index:0 ~fret:12))
;;

let tests =
  [ Alcotest.test_case "standard tuning open strings" `Quick
      standard_tuning_open_strings_are_e_a_d_g_b_e
  ; Alcotest.test_case "drop D only lowers the low string" `Quick
      drop_d_only_lowers_the_low_string
  ; Alcotest.test_case "seven string tuning has seven strings" `Quick
      seven_string_tuning_has_seven_strings
  ; Alcotest.test_case "fret 12 wraps to open pitch class" `Quick
      fret_twelve_returns_to_the_open_pitch_class
  ]
;;
