open! Base
open Fret_not

let standard_instrument =
  Instrument.create_uniform ~tuning:Tuning.standard ~fret_count:22
;;

let position string_index fret : Fretboard_position.t = { string_index; fret }

let create_rejects_a_fret_count_per_string_mismatch () =
  Alcotest.(check bool)
    "five fret counts for six strings raises" true
    (Exn.does_raise (fun () ->
         Instrument.create ~tuning:Tuning.standard
           ~fret_count:[ 22; 22; 22; 22; 22 ]))
;;

let playable_respects_per_string_fret_counts () =
  let uneven =
    Instrument.create ~tuning:Tuning.standard
      ~fret_count:[ 22; 22; 22; 22; 22; 24 ]
  in
  Alcotest.(check bool)
    "high E fret 24 playable" true
    (Instrument.playable uneven (position 5 24));
  Alcotest.(check bool)
    "B string fret 24 not playable" true
    (not (Instrument.playable uneven (position 4 24)));
  Alcotest.(check bool)
    "beyond the string count not playable" true
    (not (Instrument.playable uneven (position 6 0)));
  Alcotest.(check bool)
    "negative fret not playable" true
    (not (Instrument.playable uneven (position 0 (-1))))
;;

let max_fret_is_the_widest_string () =
  let uneven =
    Instrument.create ~tuning:Tuning.standard
      ~fret_count:[ 22; 22; 22; 22; 22; 24 ]
  in
  Alcotest.(check int) "max fret" 24 (Instrument.max_fret uneven)
;;

let semitone_at_resolves_absolute_pitch () =
  Alcotest.(check int)
    "open low E is E2" 40
    (Instrument.semitone_at standard_instrument (position 0 0));
  Alcotest.(check int)
    "high E fret 12 is E5" 76
    (Instrument.semitone_at standard_instrument (position 5 12))
;;

let range_spans_the_reachable_semitones () =
  Alcotest.(check (pair int int))
    "E2 open to E6 at fret 22" (40, 86)
    (Instrument.range standard_instrument)
;;

let tests =
  [ Alcotest.test_case "create rejects fret count mismatch" `Quick
      create_rejects_a_fret_count_per_string_mismatch
  ; Alcotest.test_case "playable respects per-string fret counts" `Quick
      playable_respects_per_string_fret_counts
  ; Alcotest.test_case "max fret is the widest string" `Quick
      max_fret_is_the_widest_string
  ; Alcotest.test_case "semitone_at resolves absolute pitch" `Quick
      semitone_at_resolves_absolute_pitch
  ; Alcotest.test_case "range spans the reachable semitones" `Quick
      range_spans_the_reachable_semitones
  ]
;;
