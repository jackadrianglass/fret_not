open! Base
open Fret_not

let c_major =
  Key.create ~tonic:(Spelled_pitch.natural Letter.C) ~quality:Key.Major
;;

let instrument =
  Instrument.create_uniform ~tuning:Tuning.standard ~fret_count:22
;;

let low_e_fret_8 : Fretboard_position.t = { string_index = 0; fret = 8 }

let as_pairs shape =
  List.map shape ~f:(fun (p : Fretboard_position.t) -> (p.string_index, p.fret))
;;

let semitone_of (p : Fretboard_position.t) = Instrument.semitone_at instrument p

let distance_is_fret_delta_plus_string_delta () =
  Alcotest.(check int)
    "3 frets + 2 strings apart" 5
    (Chunk_solver.distance
       { string_index = 0; fret = 3 }
       { string_index = 2; fret = 6 })
;;

let root_third_fifth_ascends_within_the_fret_cap () =
  let degrees =
    [ Degree_reference.natural ~degree:1 ~octave:0
    ; Degree_reference.natural ~degree:3 ~octave:0
    ; Degree_reference.natural ~degree:5 ~octave:0
    ]
  in
  let shapes =
    Chunk_solver.positions ~key:c_major ~mode:Mode.Ionian ~instrument
      ~start_anchor:low_e_fret_8 ~max_fret_distance:7 degrees
  in
  Alcotest.(check bool)
    "at least one valid shape found" true
    (not (List.is_empty shapes));
  let best = List.hd_exn shapes in
  let relative_semitones =
    List.map best ~f:(fun (p : Fretboard_position.t) -> semitone_of p)
  in
  Alcotest.(check bool)
    "strictly ascending pitch" true
    (List.is_sorted relative_semitones ~compare:Int.compare
    && not (List.contains_dup relative_semitones ~compare:Int.compare))
;;

let every_consecutive_move_stays_within_the_fret_cap () =
  let degrees =
    [ Degree_reference.natural ~degree:1 ~octave:0
    ; Degree_reference.natural ~degree:3 ~octave:0
    ; Degree_reference.natural ~degree:5 ~octave:0
    ]
  in
  let shapes =
    Chunk_solver.positions ~key:c_major ~mode:Mode.Ionian ~instrument
      ~start_anchor:low_e_fret_8 ~max_fret_distance:7 degrees
  in
  List.iter shapes ~f:(fun shape ->
      List.iter
        (List.zip_exn (low_e_fret_8 :: List.drop_last_exn shape) shape)
        ~f:(fun
            ((prev : Fretboard_position.t), (next : Fretboard_position.t)) ->
          Alcotest.(check bool)
            "no move exceeds the fret cap" true
            (Int.abs (next.fret - prev.fret) <= 7)))
;;

let shapes_are_sorted_by_ascending_total_distance () =
  let degrees =
    [ Degree_reference.natural ~degree:1 ~octave:0
    ; Degree_reference.natural ~degree:5 ~octave:0
    ]
  in
  let shapes =
    Chunk_solver.positions ~key:c_major ~mode:Mode.Ionian ~instrument
      ~start_anchor:low_e_fret_8 ~max_fret_distance:7 degrees
  in
  let totals =
    List.map shapes ~f:(fun shape ->
        let _, total =
          List.fold shape ~init:(low_e_fret_8, 0)
            ~f:(fun ((prev : Fretboard_position.t), acc) pos ->
              (pos, acc + Chunk_solver.distance prev pos))
        in
        total)
  in
  Alcotest.(check bool)
    "totals non-decreasing" true
    (List.is_sorted totals ~compare:Int.compare)
;;

let an_impossibly_tight_cap_finds_nothing () =
  (* A 0-fret cap pins every note to the exact same fret column across all
     6 strings - at most 6 distinct pitch classes are reachable there, so
     all 7 diatonic degrees can never simultaneously fit. *)
  let degrees =
    List.init 7 ~f:(fun i -> Degree_reference.natural ~degree:(i + 1) ~octave:0)
  in
  let shapes =
    Chunk_solver.positions ~key:c_major ~mode:Mode.Ionian ~instrument
      ~start_anchor:low_e_fret_8 ~max_fret_distance:0 degrees
  in
  Alcotest.(check (list (list (pair int int))))
    "no shape fits all 7 degrees into one fret column" []
    (List.map shapes ~f:as_pairs)
;;

let intervals_of shape =
  let relative_semitones =
    List.map shape ~f:(fun (p : Fretboard_position.t) -> semitone_of p)
  in
  List.map2_exn (List.drop_last_exn relative_semitones)
    (List.tl_exn relative_semitones) ~f:(fun a b -> b - a)
;;

let a_reframed_wrapped_octave_still_lands_a_real_minor_third_up () =
  (* [1; 3; 5] reframed onto root degree 6 (Chunk.reframe: degree 3 wraps to
     degree 1, octave+1; degree 5 wraps to degree 3, octave+1) should still
     resolve to A-C-E - a real minor third then a real major third, both
     ascending - not the wrapped octave being taken literally and landing a
     full 12 semitones higher than that. *)
  let degrees =
    [ Degree_reference.natural ~degree:6 ~octave:0
    ; Degree_reference.natural ~degree:1 ~octave:1
    ; Degree_reference.natural ~degree:3 ~octave:1
    ]
  in
  let shapes =
    Chunk_solver.positions ~key:c_major ~mode:Mode.Ionian ~instrument
      ~start_anchor:low_e_fret_8 ~max_fret_distance:7 degrees
  in
  Alcotest.(check bool)
    "at least one valid shape found" true
    (not (List.is_empty shapes));
  Alcotest.(check (list int))
    "minor third then major third, both ascending" [ 3; 4 ]
    (intervals_of (List.hd_exn shapes))
;;

let an_authored_octave_leap_is_preserved () =
  (* degree 1 at octave 0 then octave 1 is a full octave leap by
     construction - the solver must realize that leap exactly (12
     semitones), not silently resolve the second note to whatever's
     nearest within the fret cap. *)
  let degrees =
    [ Degree_reference.natural ~degree:1 ~octave:0
    ; Degree_reference.natural ~degree:1 ~octave:1
    ]
  in
  let shapes =
    Chunk_solver.positions ~key:c_major ~mode:Mode.Ionian ~instrument
      ~start_anchor:low_e_fret_8 ~max_fret_distance:7 degrees
  in
  Alcotest.(check bool)
    "at least one valid shape found" true
    (not (List.is_empty shapes));
  Alcotest.(check (list int))
    "a full octave, ascending" [ 12 ]
    (intervals_of (List.hd_exn shapes))
;;

let tests =
  [ Alcotest.test_case "distance is fret delta + string delta" `Quick
      distance_is_fret_delta_plus_string_delta
  ; Alcotest.test_case "root-3rd-5th ascends within the fret cap" `Quick
      root_third_fifth_ascends_within_the_fret_cap
  ; Alcotest.test_case "every consecutive move stays within the fret cap" `Quick
      every_consecutive_move_stays_within_the_fret_cap
  ; Alcotest.test_case "shapes are sorted by ascending total distance" `Quick
      shapes_are_sorted_by_ascending_total_distance
  ; Alcotest.test_case "an impossibly tight cap finds nothing" `Quick
      an_impossibly_tight_cap_finds_nothing
  ; Alcotest.test_case
      "a reframed wrapped octave still lands a real minor third up" `Quick
      a_reframed_wrapped_octave_still_lands_a_real_minor_third_up
  ; Alcotest.test_case "an authored octave leap is preserved" `Quick
      an_authored_octave_leap_is_preserved
  ]
;;
