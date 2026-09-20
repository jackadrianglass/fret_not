open! Base
open Fret_not

let c_major_pentatonic_drops_the_fourth_and_seventh () =
  Alcotest.(check (list string))
    "C major pentatonic"
    [ "C"; "D"; "E"; "G"; "A" ]
    (Test_helpers.names (Pentatonic.major ~root:Pitch_class.c))
;;

let a_minor_pentatonic_drops_the_second_and_sixth () =
  Alcotest.(check (list string))
    "A minor pentatonic"
    [ "A"; "C"; "D"; "E"; "G" ]
    (Test_helpers.names (Pentatonic.minor ~root:Pitch_class.a))
;;

let tests =
  [ Alcotest.test_case "C major pentatonic drops the 4th and 7th" `Quick
      c_major_pentatonic_drops_the_fourth_and_seventh
  ; Alcotest.test_case "A minor pentatonic drops the 2nd and 6th" `Quick
      a_minor_pentatonic_drops_the_second_and_sixth
  ]
;;
