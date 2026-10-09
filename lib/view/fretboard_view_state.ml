open! Base

type label_mode =
  | Degree_number
  | Note_name

type page =
  | Chunk_editing
  | Tab_viewing
[@@deriving eq]

type t =
  { page_index : int
  ; tonic_index : int
  ; quality_index : int
  ; tonic_dropdown_open : bool
  ; quality_dropdown_open : bool
  ; label_mode_index : int
  ; label_mode_dropdown_open : bool
  }

let initial =
  { page_index = 0
  ; tonic_index = 0
  ; quality_index = 0
  ; tonic_dropdown_open = false
  ; quality_dropdown_open = false
  ; label_mode_index = 0
  ; label_mode_dropdown_open = false
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

let page_of_index = function 0 -> Chunk_editing | _ -> Tab_viewing
let quality_of_index = function 0 -> Key.Major | _ -> Key.Minor
let label_mode_of_index = function 0 -> Degree_number | _ -> Note_name
let mode_of_quality = Key.mode_of_quality

let key t =
  Key.create
    ~tonic:(List.nth_exn tonics t.tonic_index)
    ~quality:(quality_of_index t.quality_index)
;;

let mode t = mode_of_quality (quality_of_index t.quality_index)

let highlighted_positions t ~(config : Fretboard_view_config.t) =
  Fretboard.degrees_in_window ~instrument:config.instrument ~key:(key t)
    ~mode:(mode t) ~min_fret:0
    ~max_fret:(Instrument.max_fret config.instrument)
  |> List.map ~f:fst
;;

let position_label_text ~root ~mode ~label_mode (scale_degree : Scale_degree.t)
    =
  match label_mode with
  | Degree_number -> Scale_degree.label scale_degree
  | Note_name ->
      Spelled_pitch.to_string (Scale_degree.spelled ~root ~mode scale_degree)
;;
