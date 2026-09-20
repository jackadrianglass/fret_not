open! Base
open Fret_not

let d = Pitch_class.of_int 2

let dorian_from_root_matches_rotation () =
  Alcotest.(check (list string))
    "D Dorian"
    [ "D"; "E"; "F"; "G"; "A"; "B"; "C" ]
    (Test_helpers.names (Mode.degrees Dorian ~root:d))
;;

let tests =
  [ Alcotest.test_case "D Dorian from its own root" `Quick
      dorian_from_root_matches_rotation
  ]
;;
