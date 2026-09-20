open! Base

type quality =
  | Major
  | Minor

type t =
  { tonic : Pitch_class.t
  ; quality : quality
  }

let create ~tonic ~quality = { tonic; quality }
let tonic t = t.tonic
let quality t = t.quality

let parent_major_root t =
  match t.quality with
  | Major -> t.tonic
  | Minor -> Pitch_class.add t.tonic (-Mode.root_offset_semitones Aeolian)
;;

let mode_root t mode =
  Pitch_class.add (parent_major_root t) (Mode.root_offset_semitones mode)
;;

let modes t =
  List.map Mode.all ~f:(fun m -> (m, Mode.degrees m ~root:(mode_root t m)))
;;
