open! Base

type label_mode =
  | Degree_number
  | Note_name

type scale =
  | Diatonic
  | Pentatonic
  | Arpeggio

type t =
  { tonic_index : int
  ; quality_index : int
  ; tonic_dropdown_open : bool
  ; quality_dropdown_open : bool
  ; label_mode_index : int
  ; label_mode_dropdown_open : bool
  ; scale_index : int
  ; scale_dropdown_open : bool
  ; position_index : int
  ; position_dropdown_open : bool
  }

let initial =
  { tonic_index = 0
  ; quality_index = 0
  ; tonic_dropdown_open = false
  ; quality_dropdown_open = false
  ; label_mode_index = 0
  ; label_mode_dropdown_open = false
  ; scale_index = 0
  ; scale_dropdown_open = false
  ; position_index = 0
  ; position_dropdown_open = false
  }
;;

let quality_of_index = function 0 -> Key.Major | _ -> Key.Minor
let label_mode_of_index = function 0 -> Degree_number | _ -> Note_name
let scale_of_index = function 0 -> Diatonic | 1 -> Pentatonic | _ -> Arpeggio
let mode_of_quality = Key.mode_of_quality

let key t =
  Key.create
    ~tonic:(Pitch_class.of_int t.tonic_index)
    ~quality:(quality_of_index t.quality_index)
;;

let mode t = mode_of_quality (quality_of_index t.quality_index)

let scale_degrees t =
  match scale_of_index t.scale_index with
  | Diatonic -> Scale_degree.diatonic
  | Pentatonic -> Fretboard.pentatonic_degrees ~key:(key t)
  | Arpeggio -> Fretboard.arpeggio_degrees ~key:(key t)
;;

let scale_mode t =
  match scale_of_index t.scale_index with
  | Diatonic -> mode t
  | _ -> mode_of_quality (quality_of_index t.quality_index)
;;

let scale_root t = Key.mode_root (key t) (scale_mode t)

let highlighted_positions t ~(config : Fretboard_view_config.t) =
  let key = key t in
  let anchor : Fretboard_position.t = { string_index = 0; fret = 0 } in
  match scale_of_index t.scale_index with
  | Diatonic ->
      let mode = mode t in
      Fretboard.positions_in_window ~key ~mode ~tuning:config.tuning
        ~anchor_position:anchor ~min_fret:0 ~max_fret:config.fret_count
  | Pentatonic ->
      Fretboard.pentatonic_positions_in_window ~key ~tuning:config.tuning
        ~anchor_position:anchor ~min_fret:0 ~max_fret:config.fret_count
  | Arpeggio ->
      Fretboard.arpeggio_positions_in_window ~key ~tuning:config.tuning
        ~anchor_position:anchor ~min_fret:0 ~max_fret:config.fret_count
;;

let three_notes_per_string_positions t ~(config : Fretboard_view_config.t) =
  let key = key t in
  let mode = mode t in
  Fretboard.three_notes_per_string_positions ~key ~mode ~tuning:config.tuning
;;

let two_notes_per_string_positions t ~(config : Fretboard_view_config.t) =
  let key = key t in
  Fretboard.two_notes_per_string_positions ~key ~tuning:config.tuning
;;

let one_note_per_string_positions t ~(config : Fretboard_view_config.t) =
  let key = key t in
  Fretboard.one_note_per_string_positions ~key ~tuning:config.tuning
;;

let selected_position t ~config =
  if t.position_index = 0 then None
  else
    let positions =
      match scale_of_index t.scale_index with
      | Diatonic -> three_notes_per_string_positions t ~config
      | Pentatonic -> two_notes_per_string_positions t ~config
      | Arpeggio -> one_note_per_string_positions t ~config
    in
    Some (List.nth_exn positions (t.position_index - 1))
;;

let mode_names_from mode =
  let start_index =
    List.findi_exn Mode.all ~f:(fun _ m -> Poly.equal m mode) |> fst
  in
  List.drop Mode.all start_index @ List.take Mode.all start_index
  |> List.map ~f:Mode.name
;;

let position_options t =
  let options =
    match scale_of_index t.scale_index with
    | Diatonic ->
        mode_names_from (mode t)
        |> List.mapi ~f:(fun i name -> Int.to_string (i + 1) ^ " " ^ name)
    | Pentatonic -> List.init 5 ~f:(fun i -> Int.to_string (i + 1))
    | Arpeggio -> [ "Root"; "1st Inv"; "2nd Inv" ]
  in
  "All" :: options |> String.concat ~sep:";"
;;

let position_label_text ~root ~mode ~label_mode (scale_degree : Scale_degree.t)
    =
  match label_mode with
  | Degree_number -> Scale_degree.label scale_degree
  | Note_name ->
      Pitch_class.to_string (Scale_degree.pitch_class ~root ~mode scale_degree)
;;
