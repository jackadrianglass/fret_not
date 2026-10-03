open! Base

type label_mode =
  | Degree_number
  | Note_name

type scale =
  | Diatonic
  | Pentatonic
  | Arpeggio
[@@deriving eq]

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
let scale_of_index = function 0 -> Diatonic | 1 -> Pentatonic | _ -> Arpeggio
let mode_of_quality = Key.mode_of_quality

let key t =
  Key.create
    ~tonic:(List.nth_exn tonics t.tonic_index)
    ~quality:(quality_of_index t.quality_index)
;;

let mode t = mode_of_quality (quality_of_index t.quality_index)

let scale_degrees t =
  let quality = quality_of_index t.quality_index in
  let preset ~major ~minor =
    match quality with Key.Major -> major | Key.Minor -> minor
  in
  match scale_of_index t.scale_index with
  | Diatonic -> Scale_degree.diatonic
  | Pentatonic -> preset ~major:Pentatonic.major ~minor:Pentatonic.minor
  | Arpeggio -> preset ~major:Arpeggio.major ~minor:Arpeggio.minor
;;

let notes_per_string t =
  match scale_of_index t.scale_index with
  | Diatonic -> 3
  | Pentatonic -> 2
  | Arpeggio -> 1
;;

let scale_mode t = mode t
let scale_root t = Key.mode_root (key t) (scale_mode t)

let highlighted_positions t ~(config : Fretboard_view_config.t) =
  let key = key t in
  let mode = mode t in
  let degrees = scale_degrees t in
  Fretboard.degrees_in_window ~instrument:config.instrument ~key ~mode
    ~min_fret:0
    ~max_fret:(Instrument.max_fret config.instrument)
  |> List.filter ~f:(fun (_, dr) ->
      List.mem degrees
        (Degree_reference.scale_degree dr)
        ~equal:Scale_degree.equal)
  |> List.map ~f:fst
;;

let position_shapes t ~(config : Fretboard_view_config.t) =
  Shape.positions ~instrument:config.instrument ~key:(key t) ~mode:(mode t)
    ~degrees:(scale_degrees t) ~notes_per_string:(notes_per_string t)
    ~anchor:{ Fretboard_position.string_index = 0; fret = 0 }
;;

let selected_position t ~config =
  let shapes = position_shapes t ~config in
  if t.position_index > 0 && t.position_index <= List.length shapes then
    Some (List.nth_exn shapes (t.position_index - 1))
  else None
;;

let position_options t ~config =
  let shape_count = List.length (position_shapes t ~config) in
  "All" :: List.init shape_count ~f:(fun i -> Int.to_string (i + 1))
  |> String.concat ~sep:";"
;;

let position_label_text ~root ~mode ~label_mode (scale_degree : Scale_degree.t)
    =
  match label_mode with
  | Degree_number -> Scale_degree.label scale_degree
  | Note_name ->
      Spelled_pitch.to_string (Scale_degree.spelled ~root ~mode scale_degree)
;;
