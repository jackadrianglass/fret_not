open! Base

type label_mode =
  | Degree_number
  | Note_name

type scale = Diatonic

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

let tonics =
  [ Letter.C
  ; Letter.C
  ; Letter.D
  ; Letter.D
  ; Letter.E
  ; Letter.F
  ; Letter.F
  ; Letter.G
  ; Letter.G
  ; Letter.A
  ; Letter.A
  ; Letter.B
  ]
  |> List.mapi ~f:(fun index letter ->
      Spelled_pitch.of_pitch_class_exn ~letter
        ~pitch_class:(Pitch_class.of_int index))
;;

let quality_of_index = function 0 -> Key.Major | _ -> Key.Minor
let label_mode_of_index = function 0 -> Degree_number | _ -> Note_name
let scale_of_index _ = Diatonic
let mode_of_quality = Key.mode_of_quality

let key t =
  Key.create
    ~tonic:(List.nth_exn tonics t.tonic_index)
    ~quality:(quality_of_index t.quality_index)
;;

let mode t = mode_of_quality (quality_of_index t.quality_index)
let scale_degrees t = Scale_degree.diatonic
let scale_mode t = mode t
let scale_root t = Key.mode_root (key t) (scale_mode t)

let highlighted_positions t ~(config : Fretboard_view_config.t) =
  let key = key t in
  let mode = mode t in
  Fretboard.degrees_in_window ~instrument:config.instrument ~key ~mode
    ~min_fret:0 ~max_fret:config.fret_count
  |> List.map ~f:fst
;;

let three_notes_per_string_positions t ~(config : Fretboard_view_config.t) =
  Shape.positions ~instrument:config.instrument ~key:(key t) ~mode:(mode t)
    ~degrees:Scale_degree.diatonic ~notes_per_string:3
    ~anchor:{ Fretboard_position.string_index = 0; fret = 0 }
;;

let selected_position t ~config =
  if t.position_index = 0 then None
  else
    let positions = three_notes_per_string_positions t ~config in
    Some (List.nth_exn positions (t.position_index - 1))
;;

let position_options _t = "All;1;2;3;4;5;6;7"

let position_label_text ~root ~mode ~label_mode (scale_degree : Scale_degree.t)
    =
  match label_mode with
  | Degree_number -> Scale_degree.label scale_degree
  | Note_name ->
      Spelled_pitch.to_string (Scale_degree.spelled ~root ~mode scale_degree)
;;
