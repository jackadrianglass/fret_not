open! Base

let distance (a : Fretboard_position.t) (b : Fretboard_position.t) =
  Int.abs (a.fret - b.fret) + Int.abs (a.string_index - b.string_index)
;;

(* A real, ever-increasing semitone height for this degree - its pitch class
   (mod 12) plus a full 12 semitones per octave the degree itself carries.
   Two degrees a Chunk was authored with an interval between (e.g. reframed
   onto a new chord root, per Chunk.reframe) keep that same interval here,
   since reframing only ever shifts every degree in a chunk by the same
   amount. *)
let absolute_semitone ~key ~mode (dr : Degree_reference.t) =
  Pitch_class.to_int (Fretboard.target_pitch_class ~key ~mode dr)
  + (12 * dr.octave)
;;

let octave_sweep = List.range (-4) 5

let first_note_candidates ~key ~mode ~tuning
    ~(start_anchor : Fretboard_position.t) ~max_fret_distance
    (dr : Degree_reference.t) =
  List.concat_map octave_sweep ~f:(fun octave ->
      Fretboard.to_positions ~key ~mode ~tuning ~anchor_position:start_anchor
        { dr with octave })
  |> List.filter ~f:(fun (p : Fretboard_position.t) ->
      Int.abs (p.fret - start_anchor.fret) <= max_fret_distance)
;;

let candidates_for_semitone ~tuning ~(near : Fretboard_position.t)
    ~max_fret_distance target_semitone =
  List.init (Tuning.string_count tuning) ~f:Fn.id
  |> List.filter_map ~f:(fun string_index ->
      let open_offset = Tuning.relative_semitone tuning ~string_index ~fret:0 in
      let fret = target_semitone - open_offset in
      if fret >= 0 && Int.abs (fret - near.fret) <= max_fret_distance then
        Some { Fretboard_position.string_index; fret }
      else None)
;;

let rec chain ~key ~mode ~tuning ~max_fret_distance ~offset anchor = function
  | [] -> [ [] ]
  | dr :: rest ->
      let target = absolute_semitone ~key ~mode dr + offset in
      candidates_for_semitone ~tuning ~near:anchor ~max_fret_distance target
      |> List.concat_map ~f:(fun position ->
          chain ~key ~mode ~tuning ~max_fret_distance ~offset position rest
          |> List.map ~f:(fun tail -> position :: tail))
;;

let total_distance anchor shape =
  let _, total =
    List.fold shape ~init:(anchor, 0) ~f:(fun (prev, acc) pos ->
        (pos, acc + distance prev pos))
  in
  total
;;

let positions ~key ~mode ~tuning ~start_anchor ~max_fret_distance = function
  | [] -> [ [] ]
  | first :: rest ->
      first_note_candidates ~key ~mode ~tuning ~start_anchor ~max_fret_distance
        first
      |> List.concat_map ~f:(fun (position : Fretboard_position.t) ->
          let offset =
            Tuning.relative_semitone tuning ~string_index:position.string_index
              ~fret:position.fret
            - absolute_semitone ~key ~mode first
          in
          chain ~key ~mode ~tuning ~max_fret_distance ~offset position rest
          |> List.map ~f:(fun tail -> position :: tail))
      |> List.sort ~compare:(fun a b ->
          Int.compare
            (total_distance start_anchor a)
            (total_distance start_anchor b))
;;
