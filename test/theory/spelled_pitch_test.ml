open! Base
open Fret_not

let open_ letter = Spelled_pitch.natural letter

let to_string_spells_the_accidental () =
  Alcotest.(check (list string))
    "spellings"
    [ "C#"; "Bb"; "F"; "E##"; "Cbb" ]
    [ Spelled_pitch.to_string
        (Spelled_pitch.create ~letter:Letter.C ~alteration:Alteration.Sharp)
    ; Spelled_pitch.to_string
        (Spelled_pitch.create ~letter:Letter.B ~alteration:Alteration.Flat)
    ; Spelled_pitch.to_string (open_ Letter.F)
    ; Spelled_pitch.to_string
        (Spelled_pitch.create ~letter:Letter.E
           ~alteration:Alteration.Double_sharp)
    ; Spelled_pitch.to_string
        (Spelled_pitch.create ~letter:Letter.C
           ~alteration:Alteration.Double_flat)
    ]
;;

let semitone_is_not_folded_into_an_octave () =
  (* What lets Note handle the B/C octave wrap: Cb sits a semitone below C,
     B# a semitone above B, rather than being folded to their pitch
     classes. *)
  Alcotest.(check (list int))
    "semitones" [ -1; 12 ]
    [ Spelled_pitch.semitone
        (Spelled_pitch.create ~letter:Letter.C ~alteration:Alteration.Flat)
    ; Spelled_pitch.semitone
        (Spelled_pitch.create ~letter:Letter.B ~alteration:Alteration.Sharp)
    ]
;;

let pitch_class_folds_through_the_wrap () =
  Alcotest.(check (list int))
    "pitch classes: Cb -> 11 (B), B# -> 0 (C), F# -> 6" [ 11; 0; 6 ]
    [ Pitch_class.to_int
        (Spelled_pitch.pitch_class
           (Spelled_pitch.create ~letter:Letter.C ~alteration:Alteration.Flat))
    ; Pitch_class.to_int
        (Spelled_pitch.pitch_class
           (Spelled_pitch.create ~letter:Letter.B ~alteration:Alteration.Sharp))
    ; Pitch_class.to_int
        (Spelled_pitch.pitch_class
           (Spelled_pitch.create ~letter:Letter.F ~alteration:Alteration.Sharp))
    ]
;;

let alteration_for_picks_the_smallest_representable_accidental () =
  let of_pair (letter, pitch_class) =
    Option.map
      ~f:(fun alteration -> Int.to_string (Alteration.semitones alteration))
      (Spelled_pitch.alteration_for ~letter
         ~pitch_class:(Pitch_class.of_int pitch_class))
  in
  (* G sounding as F# is spelled Gb, C sounding as B is spelled Cb, and no
     double accidental can stretch C to E. *)
  Alcotest.(check (list (option string)))
    "accidentals in semitones: -1 = flat, 0 = natural"
    [ Some "-1"; Some "-1"; Some "0"; None ]
    (List.map ~f:of_pair
       [ (Letter.G, 6); (Letter.C, 11); (Letter.F, 5); (Letter.C, 4) ])
;;

let equal_is_spelling_sensitive () =
  let c_sharp =
    Spelled_pitch.create ~letter:Letter.C ~alteration:Alteration.Sharp
  in
  let d_flat =
    Spelled_pitch.create ~letter:Letter.D ~alteration:Alteration.Flat
  in
  Alcotest.(check bool)
    "C# and Db sound the same" true
    (Pitch_class.equal
       (Spelled_pitch.pitch_class c_sharp)
       (Spelled_pitch.pitch_class d_flat));
  Alcotest.(check bool)
    "C# and Db are not the same spelling" true
    (not (Spelled_pitch.equal c_sharp d_flat))
;;

let tests =
  [ Alcotest.test_case "to_string spells the accidental" `Quick
      to_string_spells_the_accidental
  ; Alcotest.test_case "semitone is not folded into an octave" `Quick
      semitone_is_not_folded_into_an_octave
  ; Alcotest.test_case "pitch class folds through the wrap" `Quick
      pitch_class_folds_through_the_wrap
  ; Alcotest.test_case "alteration_for picks the smallest accidental" `Quick
      alteration_for_picks_the_smallest_representable_accidental
  ; Alcotest.test_case "equal is spelling sensitive" `Quick
      equal_is_spelling_sensitive
  ]
;;
