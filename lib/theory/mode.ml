open! Base

type t =
  | Ionian
  | Dorian
  | Phrygian
  | Lydian
  | Mixolydian
  | Aeolian
  | Locrian
[@@deriving eq]

let all = [ Ionian; Dorian; Phrygian; Lydian; Mixolydian; Aeolian; Locrian ]

let name = function
  | Ionian -> "Ionian"
  | Dorian -> "Dorian"
  | Phrygian -> "Phrygian"
  | Lydian -> "Lydian"
  | Mixolydian -> "Mixolydian"
  | Aeolian -> "Aeolian"
  | Locrian -> "Locrian"
;;

let major_scale_intervals = [ 0; 2; 4; 5; 7; 9; 11 ]

let rotation_index = function
  | Ionian -> 0
  | Dorian -> 1
  | Phrygian -> 2
  | Lydian -> 3
  | Mixolydian -> 4
  | Aeolian -> 5
  | Locrian -> 6
;;

let root_offset_semitones t =
  List.nth_exn major_scale_intervals (rotation_index t)
;;

let pitch_classes t ~root =
  let parent_major_root = Pitch_class.add root (-root_offset_semitones t) in
  let parent_scale =
    List.map major_scale_intervals ~f:(Pitch_class.add parent_major_root)
  in
  let idx = rotation_index t in
  List.drop parent_scale idx @ List.take parent_scale idx
;;
