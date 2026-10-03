open! Base
open Fret_not

let c = Spelled_pitch.natural Letter.C
let a = Spelled_pitch.natural Letter.A
let spelled name = Spelled_pitch.to_string name

let mode_root_spells_each_mode_of_c_major () =
  let key = Key.create ~tonic:c ~quality:Major in
  Alcotest.(check (list string))
    "mode roots of C major"
    [ "C"; "D"; "E"; "F"; "G"; "A"; "B" ]
    (List.map Mode.all ~f:(fun mode -> spelled (Key.mode_root key mode)))
;;

let mode_root_spells_each_mode_of_a_minor () =
  (* Ionian comes first in Mode.all, so a minor key's mode roots run from
     the parent major's tonic, not the minor tonic. *)
  let key = Key.create ~tonic:a ~quality:Minor in
  Alcotest.(check (list string))
    "mode roots of A minor, Ionian first"
    [ "C"; "D"; "E"; "F"; "G"; "A"; "B" ]
    (List.map Mode.all ~f:(fun mode -> spelled (Key.mode_root key mode)))
;;

let mode_root_spells_accidentals_off_the_parent_major () =
  let f_sharp_minor =
    Key.create
      ~tonic:
        (Spelled_pitch.create ~letter:Letter.F ~alteration:Alteration.Sharp)
      ~quality:Minor
  in
  Alcotest.(check string)
    "Aeolian root of F# minor is F#" "F#"
    (spelled (Key.mode_root f_sharp_minor Mode.Aeolian));
  Alcotest.(check string)
    "Ionian root of F# minor is the parent major A" "A"
    (spelled (Key.mode_root f_sharp_minor Mode.Ionian));
  let e_flat_major =
    Key.create
      ~tonic:(Spelled_pitch.create ~letter:Letter.E ~alteration:Alteration.Flat)
      ~quality:Major
  in
  Alcotest.(check string)
    "Lydian root of Eb major is Ab" "Ab"
    (spelled (Key.mode_root e_flat_major Mode.Lydian))
;;

let mode_pitch_classes_are_the_modes_scale () =
  let key = Key.create ~tonic:c ~quality:Major in
  Alcotest.(check (list string))
    "Dorian rooted on D"
    [ "D"; "E"; "F"; "G"; "A"; "B"; "C" ]
    (Test_helpers.names (Key.mode_pitch_classes key Mode.Dorian))
;;

let mode_of_quality_round_trips () =
  Alcotest.(check bool)
    "Major -> Ionian" true
    (Mode.equal (Key.mode_of_quality Key.Major) Mode.Ionian);
  Alcotest.(check bool)
    "Minor -> Aeolian" true
    (Mode.equal (Key.mode_of_quality Key.Minor) Mode.Aeolian)
;;

let tests =
  [ Alcotest.test_case "mode_root spells each mode of C major" `Quick
      mode_root_spells_each_mode_of_c_major
  ; Alcotest.test_case "mode_root spells each mode of A minor" `Quick
      mode_root_spells_each_mode_of_a_minor
  ; Alcotest.test_case "mode_root spells accidentals off the parent major"
      `Quick mode_root_spells_accidentals_off_the_parent_major
  ; Alcotest.test_case "mode_pitch_classes are the mode's scale" `Quick
      mode_pitch_classes_are_the_modes_scale
  ; Alcotest.test_case "mode_of_quality round trips" `Quick
      mode_of_quality_round_trips
  ]
;;
