open! Base

type t =
  { fret_span : int
  ; string_span : int
  }

let between (a : Fretboard_position.t) (b : Fretboard_position.t) =
  { fret_span = Int.abs (a.fret - b.fret)
  ; string_span = Int.abs (a.string_index - b.string_index)
  }
;;

let of_positions (positions : Fretboard_position.t list) =
  let frets = List.map positions ~f:(fun p -> p.fret) in
  let strings = List.map positions ~f:(fun p -> p.string_index) in
  match
    ( List.min_elt frets ~compare:Int.compare
    , List.max_elt frets ~compare:Int.compare
    , List.min_elt strings ~compare:Int.compare
    , List.max_elt strings ~compare:Int.compare )
  with
  | Some min_fret, Some max_fret, Some min_string, Some max_string ->
      Some
        { fret_span = max_fret - min_fret
        ; string_span = max_string - min_string
        }
  | _ -> None
;;

let semitone_span ~instrument (a : Fretboard_position.t)
    (b : Fretboard_position.t) =
  Int.abs
    (Instrument.semitone_at instrument a - Instrument.semitone_at instrument b)
;;

let compare a b =
  match Int.compare a.fret_span b.fret_span with
  | 0 -> Int.compare a.string_span b.string_span
  | c -> c
;;
