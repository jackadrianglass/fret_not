open! Base
open Fret_not

let c = Pitch_class.of_int 0
let d = Pitch_class.of_int 2

let diatonic_is_seven_natural_degrees () =
  Alcotest.(check (list int))
    "degrees 1..7" [ 1; 2; 3; 4; 5; 6; 7 ]
    (List.map Scale_degree.diatonic ~f:Scale_degree.degree);
  Alcotest.(check bool)
    "all natural" true
    (List.for_all Scale_degree.diatonic ~f:(fun sd ->
         Alteration.equal (Scale_degree.alteration sd) Alteration.Natural))
;;

let pitch_class_follows_the_mode_from_its_root () =
  Alcotest.(check (list string))
    "D Dorian"
    [ "D"; "E"; "F"; "G"; "A"; "B"; "C" ]
    (Test_helpers.degree_names ~root:d ~mode:Mode.Dorian Scale_degree.diatonic)
;;

let alteration_shifts_the_degree_pitch_class () =
  Alcotest.(check (list string))
    "C major chromatic degrees"
    [ "C#"; "D#"; "F#"; "G#"; "A#" ]
    (List.map [ 1; 2; 4; 5; 6 ] ~f:(fun degree ->
         Scale_degree.create ~degree ~alteration:Alteration.Sharp
         |> Scale_degree.pitch_class ~root:c ~mode:Mode.Ionian
         |> Pitch_class.to_string))
;;

let label_marks_the_alteration () =
  Alcotest.(check (list string))
    "labels"
    [ "1"; "b3"; "#4"; "##4"; "bb7" ]
    (List.map
       [ (1, Alteration.Natural)
       ; (3, Alteration.Flat)
       ; (4, Alteration.Sharp)
       ; (4, Alteration.Double_sharp)
       ; (7, Alteration.Double_flat)
       ]
       ~f:(fun (degree, alteration) ->
         Scale_degree.label (Scale_degree.create ~degree ~alteration)))
;;

let tests =
  [ Alcotest.test_case "diatonic is seven natural degrees" `Quick
      diatonic_is_seven_natural_degrees
  ; Alcotest.test_case "pitch class follows the mode from its root" `Quick
      pitch_class_follows_the_mode_from_its_root
  ; Alcotest.test_case "alteration shifts the degree pitch class" `Quick
      alteration_shifts_the_degree_pitch_class
  ; Alcotest.test_case "label marks the alteration" `Quick
      label_marks_the_alteration
  ]
;;
