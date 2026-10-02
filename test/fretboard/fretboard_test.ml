open! Base
open Fret_not

let c_major = Key.create ~tonic:(Spelled_pitch.natural Letter.C) ~quality:Major
let ionian = Mode.Ionian

let instrument =
  Instrument.create_uniform ~tuning:Tuning.standard ~fret_count:22
;;

let six_fret_instrument =
  Instrument.create_uniform ~tuning:Tuning.standard ~fret_count:6
;;

let as_pairs positions =
  List.map positions ~f:(fun (p : Fretboard_position.t) ->
      (p.string_index, p.fret))
;;

let degree_positions degree octave =
  Fretboard.degree_positions ~instrument ~key:c_major ~mode:ionian
    (Degree_reference.natural ~degree ~octave)
;;

let degree_one_octave_3_is_playable_on_two_strings () =
  Alcotest.(check (list (pair int int)))
    "C3 on the low E and A strings, ascending by string"
    [ (0, 8); (1, 3) ]
    (as_pairs (degree_positions 1 3))
;;

let degrees_below_the_instruments_range_have_no_positions () =
  Alcotest.(check (list (pair int int)))
    "C at the tonic's register 0 is below every open string" []
    (as_pairs (degree_positions 1 0))
;;

let positions_of_a_degree_all_sound_the_same_semitone () =
  let semitones =
    List.map (degree_positions 3 3) ~f:(fun (p : Fretboard_position.t) ->
        Instrument.semitone_at instrument p)
  in
  Alcotest.(check (list int))
    "every E3 position sounds 52" [ 52; 52; 52 ] semitones
;;

let degree_at_round_trips_through_positions () =
  List.iter
    [ (1, 3); (3, 3); (5, 3); (7, 2); (4, 4) ]
    ~f:(fun (degree, octave) ->
      let dr = Degree_reference.natural ~degree ~octave in
      List.iter (degree_positions degree octave) ~f:(fun position ->
          let round_trip =
            Fretboard.degree_at ~instrument ~key:c_major ~mode:ionian position
          in
          Alcotest.(check bool)
            (Printf.sprintf "degree %d octave %d round trips" degree octave)
            true
            (Option.equal Degree_reference.equal (Some dr) round_trip)))
;;

let degree_at_returns_none_off_key () =
  let chromatic_position : Fretboard_position.t =
    { string_index = 0; fret = 6 }
  in
  Alcotest.(check bool)
    "Bb on the low E string is off key in C major" true
    (Option.is_none
       (Fretboard.degree_at ~instrument ~key:c_major ~mode:ionian
          chromatic_position))
;;

let open_low_e_is_degree_3_at_octave_2 () =
  (* Tonic-anchored octaves: the octave counts tonic registers, and E2 sits
     two registers above the register C is anchored at (register 0 = C0). *)
  let open_low_e : Fretboard_position.t = { string_index = 0; fret = 0 } in
  let degree =
    Fretboard.degree_at ~instrument ~key:c_major ~mode:ionian open_low_e
    |> Option.value_exn
  in
  Alcotest.(check int) "degree" 3 (Degree_reference.degree degree);
  Alcotest.(check int) "octave" 2 (Degree_reference.octave degree)
;;

let every_string_shows_all_seven_degrees_within_twelve_frets () =
  let positions =
    Fretboard.degrees_in_window ~instrument ~key:c_major ~mode:ionian
      ~min_fret:0 ~max_fret:12
    |> List.map ~f:fst
  in
  let frets_on string_index =
    positions
    |> List.filter_map ~f:(fun (p : Fretboard_position.t) ->
        if p.string_index = string_index then Some p.fret else None)
    |> List.dedup_and_sort ~compare:Int.compare
  in
  Alcotest.(check (list int)) "low E" [ 0; 1; 3; 5; 7; 8; 10; 12 ] (frets_on 0);
  Alcotest.(check (list int)) "A" [ 0; 2; 3; 5; 7; 8; 10; 12 ] (frets_on 1);
  Alcotest.(check (list int)) "D" [ 0; 2; 3; 5; 7; 9; 10; 12 ] (frets_on 2);
  Alcotest.(check (list int)) "G" [ 0; 2; 4; 5; 7; 9; 10; 12 ] (frets_on 3);
  Alcotest.(check (list int)) "B" [ 0; 1; 3; 5; 6; 8; 10; 12 ] (frets_on 4);
  Alcotest.(check (list int)) "high E" [ 0; 1; 3; 5; 7; 8; 10; 12 ] (frets_on 5)
;;

let degrees_in_window_returns_the_degree_with_each_position () =
  let open_low_e : Fretboard_position.t = { string_index = 0; fret = 0 } in
  let entry =
    List.find_exn
      (Fretboard.degrees_in_window ~instrument ~key:c_major ~mode:ionian
         ~min_fret:0 ~max_fret:12) ~f:(fun (position, _) ->
        Fretboard_position.equal position open_low_e)
  in
  let degree = snd entry in
  Alcotest.(check int) "degree" 3 (Degree_reference.degree degree);
  Alcotest.(check int) "octave" 2 (Degree_reference.octave degree)
;;

let degrees_in_window_clamps_to_the_frets_each_string_actually_has () =
  let positions =
    Fretboard.degrees_in_window ~instrument:six_fret_instrument ~key:c_major
      ~mode:ionian ~min_fret:0 ~max_fret:12
    |> List.map ~f:fst
  in
  Alcotest.(check bool)
    "nothing beyond fret 6" true
    (List.for_all positions ~f:(fun (p : Fretboard_position.t) -> p.fret <= 6));
  Alcotest.(check (list (pair int int)))
    "low E keeps only its in-key frets up to 6"
    [ (0, 0); (0, 1); (0, 3); (0, 5) ]
    (List.filter positions ~f:(fun (p : Fretboard_position.t) ->
         p.string_index = 0)
    |> as_pairs)
;;

let tests =
  [ Alcotest.test_case "degree 1 octave 3 playable on two strings" `Quick
      degree_one_octave_3_is_playable_on_two_strings
  ; Alcotest.test_case "degrees below the instrument's range have no positions"
      `Quick degrees_below_the_instruments_range_have_no_positions
  ; Alcotest.test_case "positions of a degree all sound the same semitone"
      `Quick positions_of_a_degree_all_sound_the_same_semitone
  ; Alcotest.test_case "degree_at round trips through positions" `Quick
      degree_at_round_trips_through_positions
  ; Alcotest.test_case "degree_at returns None off key" `Quick
      degree_at_returns_none_off_key
  ; Alcotest.test_case "open low E is degree 3 at octave 2" `Quick
      open_low_e_is_degree_3_at_octave_2
  ; Alcotest.test_case "every string shows all seven degrees within 12 frets"
      `Quick every_string_shows_all_seven_degrees_within_twelve_frets
  ; Alcotest.test_case "degrees_in_window returns the degree with each position"
      `Quick degrees_in_window_returns_the_degree_with_each_position
  ; Alcotest.test_case
      "degrees_in_window clamps to the frets each string actually has" `Quick
      degrees_in_window_clamps_to_the_frets_each_string_actually_has
  ]
;;
