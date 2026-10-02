open! Base
open Fret_not

let open_notes tuning =
  List.init (Tuning.string_count tuning) ~f:(fun string_index ->
      Note.to_string (Tuning.open_note tuning ~string_index))
;;

let standard_tuning_open_strings_are_absolute_spn_notes () =
  Alcotest.(check (list string))
    "standard tuning"
    [ "E2"; "A2"; "D3"; "G3"; "B3"; "E4" ]
    (open_notes Tuning.standard)
;;

let drop_d_only_lowers_the_low_string () =
  Alcotest.(check (list string))
    "drop D"
    [ "D2"; "A2"; "D3"; "G3"; "B3"; "E4" ]
    (open_notes Tuning.drop_d)
;;

let seven_string_tuning_adds_a_low_b () =
  Alcotest.(check (list string))
    "seven string"
    [ "B1"; "E2"; "A2"; "D3"; "G3"; "B3"; "E4" ]
    (open_notes Tuning.standard_seven_string)
;;

let bass_four_is_an_octave_below_the_guitar () =
  Alcotest.(check (list string))
    "four string bass" [ "E1"; "A1"; "D2"; "G2" ]
    (open_notes Tuning.bass_four)
;;

let semitone_at_is_open_semitone_plus_fret () =
  Alcotest.(check (list int))
    "E2 open, E3 at the 12th fret, C4 on the B string's first fret"
    [ 40; 52; 60 ]
    [ Tuning.semitone_at Tuning.standard ~string_index:0 ~fret:0
    ; Tuning.semitone_at Tuning.standard ~string_index:0 ~fret:12
    ; Tuning.semitone_at Tuning.standard ~string_index:4 ~fret:1
    ]
;;

let retune_string_shifts_one_string_and_spells_minimally () =
  let retuned =
    Tuning.retune_string Tuning.standard ~string_index:1 ~semitones:(-2)
  in
  Alcotest.(check (list string))
    "A string down a whole step, others untouched"
    [ "E2"; "G2"; "D3"; "G3"; "B3"; "E4" ]
    (open_notes retuned)
;;

let tests =
  [ Alcotest.test_case "standard tuning open strings are absolute SPN notes"
      `Quick standard_tuning_open_strings_are_absolute_spn_notes
  ; Alcotest.test_case "drop D only lowers the low string" `Quick
      drop_d_only_lowers_the_low_string
  ; Alcotest.test_case "seven string tuning adds a low B" `Quick
      seven_string_tuning_adds_a_low_b
  ; Alcotest.test_case "bass four is an octave below the guitar" `Quick
      bass_four_is_an_octave_below_the_guitar
  ; Alcotest.test_case "semitone_at is open semitone plus fret" `Quick
      semitone_at_is_open_semitone_plus_fret
  ; Alcotest.test_case "retune_string shifts one string and spells minimally"
      `Quick retune_string_shifts_one_string_and_spells_minimally
  ]
;;
