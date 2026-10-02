open! Base

type t =
  { tuning : Tuning.t
  ; fret_count : int list
  }

let create ~tuning ~fret_count =
  if not (Int.equal (List.length fret_count) (Tuning.string_count tuning)) then
    raise
      (Invalid_argument
         "Instrument.create: one fret count per string is required")
  else { tuning; fret_count }
;;

let create_uniform ~tuning ~fret_count =
  { tuning
  ; fret_count = List.init (Tuning.string_count tuning) ~f:(fun _ -> fret_count)
  }
;;

let tuning t = t.tuning
let string_count t = Tuning.string_count t.tuning
let fret_count t ~string_index = List.nth_exn t.fret_count string_index

let max_fret t =
  List.max_elt t.fret_count ~compare:Int.compare |> Option.value_exn
;;

let playable t (position : Fretboard_position.t) =
  position.string_index >= 0
  && position.string_index < string_count t
  && position.fret >= 0
  && position.fret <= fret_count t ~string_index:position.string_index
;;

let semitone_at t (position : Fretboard_position.t) =
  Tuning.semitone_at t.tuning ~string_index:position.string_index
    ~fret:position.fret
;;

let pitch_class_at t (position : Fretboard_position.t) =
  Pitch_class.of_int (semitone_at t position)
;;

let range t =
  let lowest = Tuning.open_semitone t.tuning ~string_index:0 in
  let highest =
    let last = string_count t - 1 in
    Tuning.open_semitone t.tuning ~string_index:last
    + fret_count t ~string_index:last
  in
  (lowest, highest)
;;
