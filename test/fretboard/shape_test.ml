open! Base
open Fret_not

let c_major = Key.create ~tonic:(Spelled_pitch.natural Letter.C) ~quality:Major
let ionian = Mode.Ionian
let open_low_e : Fretboard_position.t = { string_index = 0; fret = 0 }

let instrument =
  Instrument.create_uniform ~tuning:Tuning.standard ~fret_count:22
;;

let as_pairs shape =
  List.map shape ~f:(fun (p : Fretboard_position.t) -> (p.string_index, p.fret))
;;

let scale_notes_cycles_degrees_with_octave_carry () =
  let notes =
    Shape.scale_notes ~degrees:Scale_degree.diatonic ~start:0 ~count:9
  in
  let degrees = List.map notes ~f:Degree_reference.degree in
  let octaves = List.map notes ~f:Degree_reference.octave in
  Alcotest.(check (list int))
    "degrees cycle 1..7,1,2"
    [ 1; 2; 3; 4; 5; 6; 7; 1; 2 ]
    degrees;
  Alcotest.(check (list int))
    "the octave carries once per cycle"
    [ 0; 0; 0; 0; 0; 0; 0; 1; 1 ]
    octaves
;;

let scale_notes_starts_midway_through_the_cycle () =
  let notes =
    Shape.scale_notes ~degrees:Scale_degree.diatonic ~start:5 ~count:3
  in
  Alcotest.(check (list int))
    "degrees 6,7,1" [ 6; 7; 1 ]
    (List.map notes ~f:Degree_reference.degree);
  Alcotest.(check (list int))
    "the carried degree keeps its octave" [ 0; 0; 1 ]
    (List.map notes ~f:Degree_reference.octave)
;;

let scale_notes_of_empty_degrees_is_empty () =
  Alcotest.(check (list int))
    "no degrees, no notes" []
    (List.map
       (Shape.scale_notes ~degrees:[] ~start:0 ~count:5)
       ~f:Degree_reference.degree)
;;

let diatonic_three_notes_per_string_shape_one_matches_the_known_shape () =
  let shapes =
    Shape.positions ~instrument ~key:c_major ~mode:ionian
      ~degrees:Scale_degree.diatonic ~notes_per_string:3 ~anchor:open_low_e
  in
  Alcotest.(check int) "seven shapes, one per degree" 7 (List.length shapes);
  Alcotest.(check (list (pair int int)))
    "C major Ionian shape 1"
    [ (0, 8)
    ; (0, 10)
    ; (0, 12)
    ; (1, 8)
    ; (1, 10)
    ; (1, 12)
    ; (2, 9)
    ; (2, 10)
    ; (2, 12)
    ; (3, 9)
    ; (3, 10)
    ; (3, 12)
    ; (4, 10)
    ; (4, 12)
    ; (4, 13)
    ; (5, 10)
    ; (5, 12)
    ; (5, 13)
    ]
    (as_pairs (List.hd_exn shapes))
;;

let diatonic_shape_three_is_dropped_to_its_lowest_playable_octave () =
  let shapes =
    Shape.positions ~instrument ~key:c_major ~mode:ionian
      ~degrees:Scale_degree.diatonic ~notes_per_string:3 ~anchor:open_low_e
  in
  Alcotest.(check (list (pair int int)))
    "shape 3, dropped an octave to stay on the neck"
    [ (0, 0)
    ; (0, 1)
    ; (0, 3)
    ; (1, 0)
    ; (1, 2)
    ; (1, 3)
    ; (2, 0)
    ; (2, 2)
    ; (2, 3)
    ; (3, 0)
    ; (3, 2)
    ; (3, 4)
    ; (4, 1)
    ; (4, 3)
    ; (4, 5)
    ; (5, 1)
    ; (5, 3)
    ; (5, 5)
    ]
    (as_pairs (List.nth_exn shapes 2))
;;

let every_shape_is_playable_on_the_instrument () =
  let shapes =
    Shape.positions ~instrument ~key:c_major ~mode:ionian
      ~degrees:Scale_degree.diatonic ~notes_per_string:3 ~anchor:open_low_e
  in
  Alcotest.(check bool)
    "every position playable" true
    (List.for_all shapes ~f:(fun shape ->
         List.for_all shape ~f:(fun position ->
             Instrument.playable instrument position)))
;;

let shapes_reaching_past_the_fret_limit_are_omitted () =
  let three_fret_instrument =
    Instrument.create_uniform ~tuning:Tuning.standard ~fret_count:3
  in
  let shapes =
    Shape.positions ~instrument:three_fret_instrument ~key:c_major ~mode:ionian
      ~degrees:Scale_degree.diatonic ~notes_per_string:3 ~anchor:open_low_e
  in
  Alcotest.(check bool)
    "shapes needing frets past 3 don't survive" true
    (List.for_all shapes ~f:(fun shape ->
         List.for_all shape ~f:(fun (p : Fretboard_position.t) -> p.fret <= 3)))
;;

let pentatonic_degrees_drive_two_notes_per_string_shapes () =
  let shapes =
    Shape.positions ~instrument ~key:c_major ~mode:ionian
      ~degrees:Pentatonic.major ~notes_per_string:2 ~anchor:open_low_e
  in
  Alcotest.(check int)
    "five shapes, one per pentatonic degree" 5 (List.length shapes);
  Alcotest.(check (list (pair int int)))
    "C major pentatonic shape 1"
    [ (0, 8)
    ; (0, 10)
    ; (1, 7)
    ; (1, 10)
    ; (2, 7)
    ; (2, 10)
    ; (3, 7)
    ; (3, 9)
    ; (4, 8)
    ; (4, 10)
    ; (5, 8)
    ; (5, 10)
    ]
    (as_pairs (List.hd_exn shapes))
;;

let arpeggio_degrees_drive_one_note_per_string_shapes () =
  let shapes =
    Shape.positions ~instrument ~key:c_major ~mode:ionian
      ~degrees:Arpeggio.major ~notes_per_string:1 ~anchor:open_low_e
  in
  Alcotest.(check int)
    "three shapes, one per arpeggio degree" 3 (List.length shapes);
  Alcotest.(check (list (pair int int)))
    "C major arpeggio root shape"
    [ (0, 8); (1, 7); (2, 5); (3, 5); (4, 5); (5, 3) ]
    (as_pairs (List.hd_exn shapes))
;;

let tests =
  [ Alcotest.test_case "scale_notes cycles degrees with octave carry" `Quick
      scale_notes_cycles_degrees_with_octave_carry
  ; Alcotest.test_case "scale_notes starts midway through the cycle" `Quick
      scale_notes_starts_midway_through_the_cycle
  ; Alcotest.test_case "scale_notes of empty degrees is empty" `Quick
      scale_notes_of_empty_degrees_is_empty
  ; Alcotest.test_case "diatonic 3nps shape 1 matches the known shape" `Quick
      diatonic_three_notes_per_string_shape_one_matches_the_known_shape
  ; Alcotest.test_case "diatonic shape 3 is dropped to its lowest octave" `Quick
      diatonic_shape_three_is_dropped_to_its_lowest_playable_octave
  ; Alcotest.test_case "every shape is playable on the instrument" `Quick
      every_shape_is_playable_on_the_instrument
  ; Alcotest.test_case "shapes reaching past the fret limit are omitted" `Quick
      shapes_reaching_past_the_fret_limit_are_omitted
  ; Alcotest.test_case "pentatonic degrees drive 2nps shapes" `Quick
      pentatonic_degrees_drive_two_notes_per_string_shapes
  ; Alcotest.test_case "arpeggio degrees drive 1nps shapes" `Quick
      arpeggio_degrees_drive_one_note_per_string_shapes
  ]
;;
