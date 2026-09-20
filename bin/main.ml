open! Base
open Fret_not

let canvas_width = 1200
let canvas_height = 400
let control_bar_height = 50
let margin = 50.

let layout : Fretboard_layout.t =
  { canvas_width = Float.of_int canvas_width
  ; canvas_height = Float.of_int canvas_height
  ; margin
  ; string_count = Tuning.string_count Tuning.standard
  ; fret_count = 12
  }
;;

type label_mode =
  | Degree_number
  | Note_name

type state =
  { tonic_index : int
  ; quality_index : int
  ; tonic_dropdown_open : bool
  ; quality_dropdown_open : bool
  ; label_mode_index : int
  ; label_mode_dropdown_open : bool
  }

let initial_state =
  { tonic_index = 0
  ; quality_index = 0
  ; tonic_dropdown_open = false
  ; quality_dropdown_open = false
  ; label_mode_index = 0
  ; label_mode_dropdown_open = false
  }
;;

let quality_of_index = function 0 -> Key.Major | _ -> Key.Minor
let label_mode_of_index = function 0 -> Degree_number | _ -> Note_name

let mode_of_quality = function
  | Key.Major -> Mode.Ionian
  | Key.Minor -> Mode.Aeolian
;;

let key_of_state state =
  Key.create
    ~tonic:(Pitch_class.of_int state.tonic_index)
    ~quality:(quality_of_index state.quality_index)
;;

let mode_of_state state = mode_of_quality (quality_of_index state.quality_index)

let scale_degrees_for state =
  let key = key_of_state state in
  let mode = mode_of_state state in
  Mode.degrees mode ~root:(Key.mode_root key mode)
;;

let highlighted_positions_for state =
  let key = key_of_state state in
  let mode = mode_of_state state in
  let anchor : Fretboard_position.t = { string_index = 0; fret = 0 } in
  Fretboard.positions_in_window ~key ~mode ~tuning:Tuning.standard
    ~anchor_position:anchor ~min_fret:0 ~max_fret:layout.fret_count
;;

let dropdown_left_text_padding = 6

let setup () =
  Raylib.init_window canvas_width
    (canvas_height + control_bar_height)
    "fret_not";
  Raylib.set_target_fps 60;
  Raygui.set_style (Raygui.Control.DropdownBox `Text_alignment)
    Raygui.TextAlignment.(to_int Left);
  (* raygui 2.2.2's GuiDropdownBox positions its text via
     GetTextBounds(DEFAULT, bounds), not GetTextBounds(DROPDOWNBOX, bounds)
     — so a DropdownBox-scoped Text_padding is silently ignored; only
     Default's Text_padding actually reaches it. Label draws via its own
     control id, so this doesn't touch the "showing" label. *)
  Raygui.set_style (Raygui.Control.Default `Text_padding)
    dropdown_left_text_padding
;;

let to_pixels x = Int.of_float x
let below_control_bar y = y + control_bar_height
let position_dot_radius = 11.
let root_halo_radius = position_dot_radius +. 3.
let label_font_size = 11
let fret_number_font_size = 14
let fret_number_gap_below_lowest_string = 6

let ui_style_color prop =
  Raylib.get_color (Raygui.get_style (Raygui.Control.Default prop))
;;

let ui_accent_fill () = ui_style_color `Base_color_pressed
let ui_accent_border () = ui_style_color `Border_color_pressed
let ui_accent_text () = ui_style_color `Text_color_pressed
let ui_text_size () = Raygui.get_style (Raygui.Control.Default `Text_size)

let ui_dropdown_arrow_padding () =
  Raygui.get_style (Raygui.Control.DropdownBox `Arrow_padding)
;;

let text_width text = Raylib.measure_text text (ui_text_size ())
let dropdown_text_to_arrow_gap = 12

let dropdown_width options =
  (* Text is left-aligned (see `setup`) so it only needs a small inset on
     the left; the arrow glyph is anchored `arrow_padding` in from the
     right edge, so the text's right edge needs to clear that plus a
     visible gap on the right, or the two collide. *)
  let widest_option_width =
    String.split options ~on:';'
    |> List.map ~f:text_width
    |> List.max_elt ~compare:Int.compare
    |> Option.value ~default:0
  in
  Float.of_int
    (dropdown_left_text_padding + widest_option_width
    + ui_dropdown_arrow_padding ()
    + dropdown_text_to_arrow_gap)
;;

let label_width text = Float.of_int (text_width text + 8)

let draw_fretboard_grid () =
  let open Raylib in
  for string_index = 0 to layout.string_count - 1 do
    let y =
      below_control_bar
        (to_pixels (Fretboard_layout.string_y layout ~string_index))
    in
    draw_line (to_pixels margin) y
      (canvas_width - to_pixels margin)
      y Color.darkgray
  done;
  let fret_number_y =
    below_control_bar
      (canvas_height - to_pixels margin + to_pixels root_halo_radius
     + fret_number_gap_below_lowest_string)
  in
  for fret = 0 to layout.fret_count do
    let x = to_pixels (Fretboard_layout.fret_line_x layout ~fret) in
    draw_line x
      (below_control_bar (to_pixels margin))
      x
      (below_control_bar (canvas_height - to_pixels margin))
      Color.lightgray;
    let label_x = to_pixels (Fretboard_layout.fret_center_x layout ~fret) in
    draw_text (Int.to_string fret) (label_x - 4) fret_number_y
      fret_number_font_size Color.darkgray
  done
;;

let position_label_text ~label_mode (scale_degree : Scale_degree.t) =
  match label_mode with
  | Degree_number -> Int.to_string scale_degree.degree
  | Note_name -> Pitch_class.to_string scale_degree.pitch_class
;;

let draw_centered_text text ~center_x ~center_y ~color =
  let open Raylib in
  let width = measure_text text label_font_size in
  draw_text text
    (center_x - (width / 2))
    (center_y - (label_font_size / 2))
    label_font_size color
;;

let draw_in_key_dot ~label_mode (scale_degree : Scale_degree.t) ~center_x
    ~center_y =
  Raylib.draw_circle center_x center_y position_dot_radius (ui_accent_fill ());
  Raylib.draw_circle_lines center_x center_y position_dot_radius
    (ui_accent_border ());
  if scale_degree.degree = 1 then
    Raylib.draw_circle_lines center_x center_y root_halo_radius
      Raylib.Color.black;
  draw_centered_text
    (position_label_text ~label_mode scale_degree)
    ~center_x ~center_y ~color:(ui_accent_text ())
;;

let draw_off_key_dot ~center_x ~center_y =
  Raylib.draw_circle_lines center_x center_y position_dot_radius
    Raylib.Color.lightgray
;;

let draw_fret_positions state =
  let in_key = highlighted_positions_for state in
  let scale_degrees = scale_degrees_for state in
  let label_mode = label_mode_of_index state.label_mode_index in
  for string_index = 0 to layout.string_count - 1 do
    for fret = 0 to layout.fret_count do
      let position : Fretboard_position.t = { string_index; fret } in
      let x, y = Fretboard_layout.position_point layout position in
      let center_x = to_pixels x
      and center_y = below_control_bar (to_pixels y) in
      if List.mem in_key position ~equal:Fretboard_position.equal then
        let pitch_class =
          Tuning.pitch_class_at Tuning.standard ~string_index ~fret
        in
        let scale_degree =
          List.find_exn scale_degrees ~f:(fun (d : Scale_degree.t) ->
              Pitch_class.equal d.pitch_class pitch_class)
        in
        draw_in_key_dot ~label_mode scale_degree ~center_x ~center_y
      else draw_off_key_dot ~center_x ~center_y
    done
  done
;;

let tonic_options = "C;C#;D;D#;E;F;F#;G;G#;A;A#;B"
let quality_options = "Major;Minor"
let label_mode_options = "Degrees;Notes"
let showing_text = "showing"
let control_bar_x = 10.
let control_bar_y = 15.
let control_height = 20.
let control_gap = 10.

let draw_controls (state : state) : state =
  let open Raylib in
  let tonic_width = dropdown_width tonic_options in
  let quality_width = dropdown_width quality_options in
  let showing_width = label_width showing_text in
  let label_mode_width = dropdown_width label_mode_options in
  let tonic_x = control_bar_x in
  let quality_x = tonic_x +. tonic_width +. control_gap in
  let showing_x = quality_x +. quality_width +. control_gap in
  let label_mode_x = showing_x +. showing_width +. control_gap in
  let quality_index, quality_toggled =
    Raygui.dropdown_box
      (Rectangle.create quality_x control_bar_y quality_width control_height)
      quality_options state.quality_index state.quality_dropdown_open
  in
  let quality_dropdown_open =
    if quality_toggled then not state.quality_dropdown_open
    else state.quality_dropdown_open
  in
  Raygui.label
    (Rectangle.create showing_x control_bar_y showing_width control_height)
    showing_text;
  let label_mode_index, label_mode_toggled =
    Raygui.dropdown_box
      (Rectangle.create label_mode_x control_bar_y label_mode_width
         control_height)
      label_mode_options state.label_mode_index state.label_mode_dropdown_open
  in
  let label_mode_dropdown_open =
    if label_mode_toggled then not state.label_mode_dropdown_open
    else state.label_mode_dropdown_open
  in
  let tonic_index, tonic_toggled =
    Raygui.dropdown_box
      (Rectangle.create tonic_x control_bar_y tonic_width control_height)
      tonic_options state.tonic_index state.tonic_dropdown_open
  in
  let tonic_dropdown_open =
    if tonic_toggled then not state.tonic_dropdown_open
    else state.tonic_dropdown_open
  in
  { tonic_index
  ; quality_index
  ; tonic_dropdown_open
  ; quality_dropdown_open
  ; label_mode_index
  ; label_mode_dropdown_open
  }
;;

let rec loop (state : state) =
  match Raylib.window_should_close () with
  | true -> Raylib.close_window ()
  | false ->
      let open Raylib in
      begin_drawing ();
      clear_background Color.raywhite;
      draw_fretboard_grid ();
      draw_fret_positions state;
      let next_state = draw_controls state in
      end_drawing ();
      loop next_state
;;

let () =
  setup ();
  loop initial_state
;;
