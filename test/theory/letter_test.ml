open! Base
open Fret_not

let naturals_are_not_the_full_chromatic_set () =
  Alcotest.(check (list int))
    "natural letters" [ 0; 2; 4; 5; 7; 9; 11 ]
    (List.map Letter.all ~f:(fun letter ->
         Pitch_class.to_int (Letter.to_pitch_class letter)))
;;

let offset_wraps_around_the_letter_cycle () =
  Alcotest.(check (list string))
    "F stepped forward six letters"
    [ "G"; "A"; "B"; "C"; "D"; "E" ]
    (List.init 6 ~f:(fun i -> Letter.to_string (Letter.offset Letter.F (i + 1))))
;;

let negative_offset_also_wraps () =
  Alcotest.(check string)
    "C stepped back one letter" "B"
    (Letter.to_string (Letter.offset Letter.C (-1)))
;;

let of_index_round_trips_through_index () =
  List.iter Letter.all ~f:(fun letter ->
      Alcotest.(check bool)
        (Letter.to_string letter ^ " round trips")
        true
        (Letter.equal letter (Letter.of_index (Letter.index letter))))
;;

let tests =
  [ Alcotest.test_case "naturals are not the full chromatic set" `Quick
      naturals_are_not_the_full_chromatic_set
  ; Alcotest.test_case "offset wraps around the letter cycle" `Quick
      offset_wraps_around_the_letter_cycle
  ; Alcotest.test_case "negative offset also wraps" `Quick
      negative_offset_also_wraps
  ; Alcotest.test_case "of_index round trips through index" `Quick
      of_index_round_trips_through_index
  ]
;;
