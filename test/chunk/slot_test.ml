open! Base
open Fret_not

let rest_equals_rest () =
  Alcotest.(check bool) "Rest = Rest" true (Slot.equal Rest Rest)
;;

let note_and_rest_are_never_equal () =
  let note =
    Slot.Note { Degree_reference.degree = 1; octave = 0; alteration = 0 }
  in
  Alcotest.(check bool) "Note <> Rest" false (Slot.equal note Rest)
;;

let notes_with_equal_degree_references_are_equal () =
  let dr : Degree_reference.t = { degree = 3; octave = 1; alteration = -1 } in
  Alcotest.(check bool)
    "Note dr = Note dr" true
    (Slot.equal (Slot.Note dr) (Slot.Note dr))
;;

let notes_with_different_degree_references_are_not_equal () =
  let a : Degree_reference.t = { degree = 3; octave = 1; alteration = 0 } in
  let b : Degree_reference.t = { degree = 3; octave = 1; alteration = -1 } in
  Alcotest.(check bool)
    "Note a <> Note b" false
    (Slot.equal (Slot.Note a) (Slot.Note b))
;;

let tests =
  [ Alcotest.test_case "Rest equals Rest" `Quick rest_equals_rest
  ; Alcotest.test_case "Note and Rest are never equal" `Quick
      note_and_rest_are_never_equal
  ; Alcotest.test_case "Notes with equal degree references are equal" `Quick
      notes_with_equal_degree_references_are_equal
  ; Alcotest.test_case "Notes with different degree references are not equal"
      `Quick notes_with_different_degree_references_are_not_equal
  ]
;;
