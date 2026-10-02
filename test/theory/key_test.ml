open! Base
open Fret_not

let c = Spelled_pitch.natural Letter.C
let a = Spelled_pitch.natural Letter.A

let c_major_key_produces_seven_modes_from_its_own_degrees () =
  let modes = Key.modes (Key.create ~tonic:c ~quality:Major) in
  Alcotest.(check int) "seven modes" 7 (List.length modes);
  let expected =
    [ (Mode.Ionian, [ "C"; "D"; "E"; "F"; "G"; "A"; "B" ])
    ; (Dorian, [ "D"; "E"; "F"; "G"; "A"; "B"; "C" ])
    ; (Phrygian, [ "E"; "F"; "G"; "A"; "B"; "C"; "D" ])
    ; (Lydian, [ "F"; "G"; "A"; "B"; "C"; "D"; "E" ])
    ; (Mixolydian, [ "G"; "A"; "B"; "C"; "D"; "E"; "F" ])
    ; (Aeolian, [ "A"; "B"; "C"; "D"; "E"; "F"; "G" ])
    ; (Locrian, [ "B"; "C"; "D"; "E"; "F"; "G"; "A" ])
    ]
  in
  List.iter2_exn modes expected
    ~f:(fun (mode, degrees) (expected_mode, expected_names) ->
      Alcotest.(check string)
        "mode name" (Mode.name expected_mode) (Mode.name mode);
      Alcotest.(check (list string))
        (Mode.name mode) expected_names
        (Test_helpers.names degrees))
;;

let a_minor_key_shares_c_majors_modes () =
  let c_major_modes = Key.modes (Key.create ~tonic:c ~quality:Major) in
  let a_minor_modes = Key.modes (Key.create ~tonic:a ~quality:Minor) in
  List.iter2_exn c_major_modes a_minor_modes
    ~f:(fun (_, c_degrees) (_, a_degrees) ->
      Alcotest.(check (list string))
        "relative major/minor share modes"
        (Test_helpers.names c_degrees)
        (Test_helpers.names a_degrees))
;;

let mode_with_root_finds_the_mode_rooted_on_a_pitch_class () =
  let key = Key.create ~tonic:c ~quality:Major in
  Alcotest.(check (option string))
    "D is Dorian's root in C major"
    (Some (Mode.name Dorian))
    (Option.map (Key.mode_with_root key (Pitch_class.of_int 2)) ~f:Mode.name)
;;

let mode_with_root_returns_none_for_a_pitch_class_no_mode_roots_on () =
  let key = Key.create ~tonic:c ~quality:Major in
  Alcotest.(check bool)
    "C# roots no mode of C major" true
    (Option.is_none (Key.mode_with_root key (Pitch_class.of_int 1)))
;;

let tests =
  [ Alcotest.test_case "C major key produces seven modes" `Quick
      c_major_key_produces_seven_modes_from_its_own_degrees
  ; Alcotest.test_case "A minor shares C major's modes" `Quick
      a_minor_key_shares_c_majors_modes
  ; Alcotest.test_case "mode_with_root finds the mode rooted on a pitch class"
      `Quick mode_with_root_finds_the_mode_rooted_on_a_pitch_class
  ; Alcotest.test_case "mode_with_root returns None off-scale" `Quick
      mode_with_root_returns_none_for_a_pitch_class_no_mode_roots_on
  ]
;;
