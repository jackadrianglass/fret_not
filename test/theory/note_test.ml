open! Base
open Fret_not

let c = Letter.C

let note ?(alteration = Alteration.Natural) letter octave =
  Note.of_parts ~letter ~alteration ~octave
;;

let semitone_matches_scientific_pitch_notation () =
  (* Middle C is C4 and A440 is A4; B#3 sounds as C4 and Cb4 as B3. *)
  Alcotest.(check (list int))
    "semitones" [ 60; 69; 40; 60; 59; 12 ]
    [ Note.semitone (note c 4)
    ; Note.semitone (note Letter.A 4)
    ; Note.semitone (note Letter.E 2)
    ; Note.semitone (note ~alteration:Alteration.Sharp Letter.B 3)
    ; Note.semitone (note ~alteration:Alteration.Flat c 4)
    ; Note.semitone (note c 0)
    ]
;;

let to_string_round_trips_the_common_cases () =
  Alcotest.(check (list string))
    "names"
    [ "C4"; "E2"; "C#4"; "Eb3"; "F##2" ]
    [ Note.to_string (note c 4)
    ; Note.to_string (note Letter.E 2)
    ; Note.to_string (note ~alteration:Alteration.Sharp c 4)
    ; Note.to_string (note ~alteration:Alteration.Flat Letter.E 3)
    ; Note.to_string (note ~alteration:Alteration.Double_sharp Letter.F 2)
    ]
;;

let equal_is_spelling_sensitive_but_compare_breaks_ties_by_sound () =
  let c_sharp_4 = note ~alteration:Alteration.Sharp c 4 in
  let d_flat_4 = note ~alteration:Alteration.Flat Letter.D 4 in
  Alcotest.(check bool)
    "C#4 and Db4 are distinct spellings" true
    (not (Note.equal c_sharp_4 d_flat_4));
  Alcotest.(check bool)
    "C#4 sorts before Db4 on the spelling tiebreak" true
    (Int.equal (Note.compare c_sharp_4 d_flat_4) (-1))
;;

let compare_orders_by_sounding_semitone () =
  let b3 = note Letter.B 3 in
  let c4 = note c 4 in
  let c_sharp_4 = note ~alteration:Alteration.Sharp c 4 in
  Alcotest.(check bool)
    "B3 < C#4 < C4 is false; B3 < C4 < C#4" true
    (Int.equal (Note.compare b3 c4) (-1)
    && Int.equal (Note.compare c4 c_sharp_4) (-1)
    && Int.equal (Note.compare c_sharp_4 b3) 1)
;;

let pitch_class_folds_through_the_octave_wrap () =
  Alcotest.(check bool)
    "B#3 sounds as C" true
    (Pitch_class.equal Pitch_class.c
       (Note.pitch_class (note ~alteration:Alteration.Sharp Letter.B 3)))
;;

let shift_octave_moves_the_register () =
  Alcotest.(check string)
    "E2 shifted up an octave" "E3"
    (Note.to_string (Note.shift_octave (note Letter.E 2) 1))
;;

let tests =
  [ Alcotest.test_case "semitone matches SPN" `Quick
      semitone_matches_scientific_pitch_notation
  ; Alcotest.test_case "to_string round trips the common cases" `Quick
      to_string_round_trips_the_common_cases
  ; Alcotest.test_case "equal is spelling sensitive" `Quick
      equal_is_spelling_sensitive_but_compare_breaks_ties_by_sound
  ; Alcotest.test_case "compare orders by sounding semitone" `Quick
      compare_orders_by_sounding_semitone
  ; Alcotest.test_case "pitch class folds through the octave wrap" `Quick
      pitch_class_folds_through_the_octave_wrap
  ; Alcotest.test_case "shift_octave moves the register" `Quick
      shift_octave_moves_the_register
  ]
;;
