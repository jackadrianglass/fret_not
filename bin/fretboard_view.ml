open! Base
open Fret_not

let layout (config : Fretboard_view_config.t) : Fretboard_layout.t =
  { canvas_width = Float.of_int config.canvas_width
  ; canvas_height = Float.of_int config.canvas_height
  ; margin = config.margin
  ; string_count = Tuning.string_count config.tuning
  ; fret_count = config.fret_count
  }
;;

let setup (config : Fretboard_view_config.t) =
  Raygui.set_style (Raygui.Control.DropdownBox `Text_alignment)
    Raygui.TextAlignment.(to_int Left);
  (* raygui 2.2.2's GuiDropdownBox positions its text via
     GetTextBounds(DEFAULT, bounds), not GetTextBounds(DROPDOWNBOX, bounds)
     — so a DropdownBox-scoped Text_padding is silently ignored; only
     Default's Text_padding actually reaches it. Label draws via its own
     control id, so this doesn't touch the "showing" label. *)
  Raygui.set_style (Raygui.Control.Default `Text_padding)
    config.dropdown_left_text_padding
;;

let to_pixels x = Int.of_float x

(* Adds a pixel offset to a coordinate - used for both the vertical top_y
   (stacking below the tab view) and the horizontal offset_x (centering the
   whole fixed-size content block within a larger, full-screen window). *)
let shift offset value = value + offset

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

let dropdown_width (config : Fretboard_view_config.t) options =
  let widest_option_width =
    String.split options ~on:';'
    |> List.map ~f:text_width
    |> List.max_elt ~compare:Int.compare
    |> Option.value ~default:0
  in
  Row_layout.dropdown_width ~left_text_padding:config.dropdown_left_text_padding
    ~widest_option_text_width:widest_option_width
    ~arrow_padding:(ui_dropdown_arrow_padding ())
    ~text_to_arrow_gap:config.dropdown_text_to_arrow_gap
;;

let label_width text = Float.of_int (text_width text + 8)

let draw_fretboard_grid (config : Fretboard_view_config.t) ~top_y ~offset_x =
  let open Raylib in
  let layout = layout config in
  for string_index = 0 to layout.string_count - 1 do
    let y =
      shift top_y (to_pixels (Fretboard_layout.string_y layout ~string_index))
    in
    draw_line
      (shift offset_x (to_pixels config.margin))
      y
      (shift offset_x (config.canvas_width - to_pixels config.margin))
      y Color.darkgray
  done;
  let fret_number_y =
    shift top_y
      (config.canvas_height - to_pixels config.margin
      + to_pixels config.root_halo_radius
      + config.fret_number_gap_below_lowest_string)
  in
  for fret = 0 to layout.fret_count do
    let x =
      shift offset_x (to_pixels (Fretboard_layout.fret_line_x layout ~fret))
    in
    draw_line x
      (shift top_y (to_pixels config.margin))
      x
      (shift top_y (config.canvas_height - to_pixels config.margin))
      Color.lightgray;
    let label_x =
      shift offset_x (to_pixels (Fretboard_layout.fret_center_x layout ~fret))
    in
    draw_text (Int.to_string fret) (label_x - 4) fret_number_y
      config.fret_number_font_size Color.darkgray
  done
;;

let draw_centered_text (config : Fretboard_view_config.t) text ~center_x
    ~center_y ~color =
  let open Raylib in
  let width = measure_text text config.label_font_size in
  draw_text text
    (center_x - (width / 2))
    (center_y - (config.label_font_size / 2))
    config.label_font_size color
;;

let draw_in_key_dot (config : Fretboard_view_config.t) ~label_mode ~dim
    (scale_degree : Scale_degree.t) ~center_x ~center_y =
  let shade color =
    if dim then Raylib.fade color config.dimmed_alpha else color
  in
  Raylib.draw_circle center_x center_y config.position_dot_radius
    (shade (ui_accent_fill ()));
  Raylib.draw_circle_lines center_x center_y config.position_dot_radius
    (shade (ui_accent_border ()));
  if scale_degree.degree = 1 then
    Raylib.draw_circle_lines center_x center_y config.root_halo_radius
      (shade Raylib.Color.black);
  draw_centered_text config
    (Fretboard_view_state.position_label_text ~label_mode scale_degree)
    ~center_x ~center_y
    ~color:(shade (ui_accent_text ()))
;;

let draw_off_key_dot (config : Fretboard_view_config.t) ~center_x ~center_y =
  Raylib.draw_circle_lines center_x center_y config.position_dot_radius
    Raylib.Color.lightgray
;;

let draw_fret_positions (config : Fretboard_view_config.t)
    (state : Fretboard_view_state.t) ~top_y ~offset_x =
  let layout = layout config in
  let in_key = Fretboard_view_state.highlighted_positions state ~config in
  let selected_position =
    Fretboard_view_state.selected_position state ~config
  in
  let scale_degrees = Fretboard_view_state.scale_degrees state in
  let label_mode =
    Fretboard_view_state.label_mode_of_index state.label_mode_index
  in
  for string_index = 0 to layout.string_count - 1 do
    for fret = 0 to layout.fret_count do
      let position : Fretboard_position.t = { string_index; fret } in
      let x, y = Fretboard_layout.position_point layout position in
      let center_x = shift offset_x (to_pixels x)
      and center_y = shift top_y (to_pixels y) in
      if List.mem in_key position ~equal:Fretboard_position.equal then
        let pitch_class =
          Tuning.pitch_class_at config.tuning ~string_index ~fret
        in
        let scale_degree =
          List.find_exn scale_degrees ~f:(fun (d : Scale_degree.t) ->
              Pitch_class.equal d.pitch_class pitch_class)
        in
        let dim =
          match selected_position with
          | None -> false
          | Some positions ->
              not (List.mem positions position ~equal:Fretboard_position.equal)
        in
        draw_in_key_dot config ~label_mode ~dim scale_degree ~center_x ~center_y
      else draw_off_key_dot config ~center_x ~center_y
    done
  done
;;

let tonic_options = "C;C#;D;D#;E;F;F#;G;G#;A;A#;B"
let quality_options = "Major;Minor"
let label_mode_options = "Degrees;Notes"
let showing_text = "showing"
let scale_options = "Diatonic;Pentatonic;Arpeggio"

let draw_controls (config : Fretboard_view_config.t)
    (state : Fretboard_view_state.t) ~offset_x ~offset_y :
    Fretboard_view_state.t =
  let open Raylib in
  let control_bar_x = config.control_bar_x +. Float.of_int offset_x in
  let control_bar_y = config.control_bar_y +. Float.of_int offset_y in
  let position_options = Fretboard_view_state.position_options state in
  let tonic_width = dropdown_width config tonic_options in
  let quality_width = dropdown_width config quality_options in
  let scale_width = dropdown_width config scale_options in
  let position_width = dropdown_width config position_options in
  let showing_width = label_width showing_text in
  let label_mode_width = dropdown_width config label_mode_options in
  let xs =
    Row_layout.x_positions ~start_x:control_bar_x ~gap:config.control_gap
      [ tonic_width
      ; quality_width
      ; scale_width
      ; position_width
      ; showing_width
      ; label_mode_width
      ]
  in
  let tonic_x = List.nth_exn xs 0
  and quality_x = List.nth_exn xs 1
  and scale_x = List.nth_exn xs 2
  and position_x = List.nth_exn xs 3
  and showing_x = List.nth_exn xs 4
  and label_mode_x = List.nth_exn xs 5 in
  let quality_index, quality_toggled =
    Raygui.dropdown_box
      (Rectangle.create quality_x control_bar_y quality_width
         config.control_height)
      quality_options state.quality_index state.quality_dropdown_open
  in
  let quality_dropdown_open =
    if quality_toggled then not state.quality_dropdown_open
    else state.quality_dropdown_open
  in
  let scale_index, scale_toggled =
    Raygui.dropdown_box
      (Rectangle.create scale_x control_bar_y scale_width config.control_height)
      scale_options state.scale_index state.scale_dropdown_open
  in
  let scale_dropdown_open =
    if scale_toggled then not state.scale_dropdown_open
    else state.scale_dropdown_open
  in
  let position_index, position_toggled =
    Raygui.dropdown_box
      (Rectangle.create position_x control_bar_y position_width
         config.control_height)
      position_options state.position_index state.position_dropdown_open
  in
  let position_dropdown_open =
    if position_toggled then not state.position_dropdown_open
    else state.position_dropdown_open
  in
  let position_index =
    (* Diatonic and Pentatonic have different-length position lists, so a
       stale index carried across a scale switch could point at the wrong
       position or fall outside the new list. *)
    if scale_index <> state.scale_index then 0 else position_index
  in
  Raygui.label
    (Rectangle.create showing_x control_bar_y showing_width
       config.control_height)
    showing_text;
  let label_mode_index, label_mode_toggled =
    Raygui.dropdown_box
      (Rectangle.create label_mode_x control_bar_y label_mode_width
         config.control_height)
      label_mode_options state.label_mode_index state.label_mode_dropdown_open
  in
  let label_mode_dropdown_open =
    if label_mode_toggled then not state.label_mode_dropdown_open
    else state.label_mode_dropdown_open
  in
  let tonic_index, tonic_toggled =
    Raygui.dropdown_box
      (Rectangle.create tonic_x control_bar_y tonic_width config.control_height)
      tonic_options state.tonic_index state.tonic_dropdown_open
  in
  let tonic_dropdown_open =
    if tonic_toggled then not state.tonic_dropdown_open
    else state.tonic_dropdown_open
  in
  { Fretboard_view_state.tonic_index
  ; quality_index
  ; tonic_dropdown_open
  ; quality_dropdown_open
  ; label_mode_index
  ; label_mode_dropdown_open
  ; scale_index
  ; scale_dropdown_open
  ; position_index
  ; position_dropdown_open
  }
;;
