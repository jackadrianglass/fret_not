open! Base
open Fret_not

let c_major_arpeggio_is_the_major_triad () =
  Alcotest.(check (list string))
    "C major arpeggio" [ "C"; "E"; "G" ]
    (Test_helpers.degree_names ~root:Pitch_class.c ~mode:Mode.Ionian
       Arpeggio.major)
;;

let a_minor_arpeggio_is_the_minor_triad () =
  Alcotest.(check (list string))
    "A minor arpeggio" [ "A"; "C"; "E" ]
    (Test_helpers.degree_names ~root:Pitch_class.a ~mode:Mode.Aeolian
       Arpeggio.minor)
;;

let tests =
  [ Alcotest.test_case "C major arpeggio is the major triad" `Quick
      c_major_arpeggio_is_the_major_triad
  ; Alcotest.test_case "A minor arpeggio is the minor triad" `Quick
      a_minor_arpeggio_is_the_minor_triad
  ]
;;
