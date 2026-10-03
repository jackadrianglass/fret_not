open! Base
open Fret_not

let c = Spelled_pitch.natural Letter.C
let dr degree octave = Degree_reference.natural ~degree ~octave

let altered degree alteration octave =
  Degree_reference.create
    ~scale_degree:(Scale_degree.create ~degree ~alteration)
    ~octave
;;

let interval_counts_semitones_above_the_tonic () =
  let root = Pitch_class.c in
  Alcotest.(check (list int))
    "intervals of the C major degrees at octave 0" [ 0; 2; 4; 5; 7; 9; 11 ]
    (List.map Scale_degree.diatonic ~f:(fun (d : Scale_degree.t) ->
         Degree_reference.interval ~root ~mode:Mode.Ionian
           (Degree_reference.create ~scale_degree:d ~octave:0)))
;;

let interval_anchors_the_octave_at_the_tonic () =
  Alcotest.(check int)
    "degree 1 octave 1 is 12 above the tonic" 12
    (Degree_reference.interval ~root:Pitch_class.c ~mode:Mode.Ionian (dr 1 1));
  Alcotest.(check int)
    "flat 1 of octave 0 sits under the octave above" 11
    (Degree_reference.interval ~root:Pitch_class.c ~mode:Mode.Ionian
       (altered 1 Alteration.Flat 0));
  Alcotest.(check int)
    "interval is relative to any root" 11
    (Degree_reference.interval ~root:Pitch_class.a ~mode:Mode.Ionian (dr 7 0))
;;

let note_resolves_to_spn_relative_to_a_root_note () =
  let c4 = Note.natural ~letter:Letter.C ~octave:4 in
  Alcotest.(check (list string))
    "C major degrees anchored at C4"
    [ "C4"; "D4"; "E4"; "F4"; "G4"; "A4"; "B4" ]
    (List.map Scale_degree.diatonic ~f:(fun (d : Scale_degree.t) ->
         Note.to_string
           (Degree_reference.note ~root:c4 ~mode:Mode.Ionian
              (Degree_reference.create ~scale_degree:d ~octave:0))));
  Alcotest.(check string)
    "degree 1 octave 1 is the octave above" "C5"
    (Note.to_string (Degree_reference.note ~root:c4 ~mode:Mode.Ionian (dr 1 1)))
;;

let note_spells_alterations_and_wraps_the_octave () =
  let c4 = Note.natural ~letter:Letter.C ~octave:4 in
  Alcotest.(check string)
    "flat 3 of C major is spelled Eb" "Eb4"
    (Note.to_string
       (Degree_reference.note ~root:c4 ~mode:Mode.Ionian
          (altered 3 Alteration.Flat 0)));
  Alcotest.(check string)
    "flat 1 is spelled Cb and sounds as B" "Cb5"
    (Note.to_string
       (Degree_reference.note ~root:c4 ~mode:Mode.Ionian
          (altered 1 Alteration.Flat 0)));
  let e3 = Note.natural ~letter:Letter.E ~octave:3 in
  Alcotest.(check string)
    "third of E major is spelled G#" "G#3"
    (Note.to_string (Degree_reference.note ~root:e3 ~mode:Mode.Ionian (dr 3 0)))
;;

let tests =
  [ Alcotest.test_case "interval counts semitones above the tonic" `Quick
      interval_counts_semitones_above_the_tonic
  ; Alcotest.test_case "interval anchors the octave at the tonic" `Quick
      interval_anchors_the_octave_at_the_tonic
  ; Alcotest.test_case "note resolves to SPN relative to a root note" `Quick
      note_resolves_to_spn_relative_to_a_root_note
  ; Alcotest.test_case "note spells alterations and wraps the octave" `Quick
      note_spells_alterations_and_wraps_the_octave
  ]
;;
