open! Base

type quality =
  | Major
  | Minor

type t =
  { tonic : Spelled_pitch.t
  ; quality : quality
  }

let create ~tonic ~quality = { tonic; quality }
let tonic t = t.tonic
let quality t = t.quality
let tonic_pitch_class t = Spelled_pitch.pitch_class t.tonic

let parent_major_root_pitch_class t =
  match t.quality with
  | Major -> tonic_pitch_class t
  | Minor ->
      Pitch_class.add (tonic_pitch_class t)
        (-Mode.root_offset_semitones Mode.Aeolian)
;;

let parent_major_root t =
  match t.quality with
  | Major -> t.tonic
  | Minor ->
      let letter = Letter.offset (Spelled_pitch.letter t.tonic) 2 in
      Spelled_pitch.of_pitch_class_exn ~letter
        ~pitch_class:(parent_major_root_pitch_class t)
;;

let mode_of_quality = function Major -> Mode.Ionian | Minor -> Mode.Aeolian

let mode_root t mode =
  let letter =
    Letter.offset
      (Spelled_pitch.letter (parent_major_root t))
      (Mode.rotation_index mode)
  in
  let pitch_class =
    Pitch_class.add
      (parent_major_root_pitch_class t)
      (Mode.root_offset_semitones mode)
  in
  Spelled_pitch.of_pitch_class_exn ~letter ~pitch_class
;;

let modes t =
  List.map Mode.all ~f:(fun m ->
      (m, Mode.pitch_classes m ~root:(Spelled_pitch.pitch_class (mode_root t m))))
;;

let mode_with_root t pitch_class =
  List.find_map (modes t) ~f:(fun (m, pitch_classes) ->
      if Pitch_class.equal (List.hd_exn pitch_classes) pitch_class then Some m
      else None)
;;
