open! Base

type t = Pitch_class.t list

let create open_string_pitch_classes = open_string_pitch_classes
let string_count t = List.length t

let pitch_class_at t ~string_index ~fret =
  Pitch_class.add (List.nth_exn t string_index) fret
;;

let relative_semitone t ~string_index ~fret =
  let string_offsets =
    List.folding_map t ~init:(0, None)
      ~f:(fun (cumulative, previous) pitch_class ->
        let cumulative =
          match previous with
          | None -> 0
          | Some previous_pitch_class ->
              cumulative
              + (Pitch_class.to_int pitch_class
                - Pitch_class.to_int previous_pitch_class
                + 12)
                % 12
        in
        ((cumulative, Some pitch_class), cumulative))
  in
  List.nth_exn string_offsets string_index + fret
;;

let standard = create Pitch_class.[ e; a; d; g; b; e ]
let drop_d = create Pitch_class.[ d; a; d; g; b; e ]
let standard_seven_string = create Pitch_class.[ b; e; a; d; g; b; e ]
