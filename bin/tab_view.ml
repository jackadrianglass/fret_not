open! Base
open Fret_not

let to_pixels x = Int.of_float x

let layout (config : Tab_view_config.t) ~canvas_width ~string_count ~note_count
    : Tab_layout.t =
  { canvas_width = Float.of_int canvas_width
  ; margin = config.margin
  ; string_count
  ; row_spacing = config.row_spacing
  ; note_count
  }
;;

let string_label instrument ~string_index =
  Note.to_string (Tuning.open_note (Instrument.tuning instrument) ~string_index)
;;

let draw_string_lines (config : Tab_view_config.t) (layout : Tab_layout.t)
    ~canvas_width ~instrument ~top_y ~offset_x =
  let open Raylib in
  for string_index = 0 to layout.string_count - 1 do
    let y = top_y + to_pixels (Tab_layout.string_y layout ~string_index) in
    draw_text
      (string_label instrument ~string_index)
      (4 + offset_x)
      (y - (config.string_label_font_size / 2))
      config.string_label_font_size Color.darkgray;
    draw_line
      (offset_x + to_pixels layout.margin)
      y
      (offset_x + canvas_width - to_pixels layout.margin)
      y Color.lightgray
  done
;;

let draw_notes (config : Tab_view_config.t) (layout : Tab_layout.t) ~top_y
    ~offset_x (positions : Fretboard_position.t list) =
  let open Raylib in
  List.iteri positions ~f:(fun note_index (position : Fretboard_position.t) ->
      let x = offset_x + to_pixels (Tab_layout.note_x layout ~note_index) in
      let y =
        top_y
        + to_pixels
            (Tab_layout.string_y layout ~string_index:position.string_index)
      in
      let text = Int.to_string position.fret in
      let width = measure_text text config.fret_number_font_size in
      draw_text text
        (x - (width / 2))
        (y - (config.fret_number_font_size / 2))
        config.fret_number_font_size Color.black)
;;

let draw_rule (config : Tab_view_config.t) ~canvas_width ~y ~offset_x =
  (* Full content width (not the string lines' margin-to-margin span) and a
     thick filled bar (not a hairline) so this reads as a section divider,
     not a 7th string. *)
  Raylib.draw_rectangle offset_x y canvas_width
    (to_pixels config.rule_thickness)
    Raylib.Color.black
;;

let draw (config : Tab_view_config.t) ~canvas_width ~instrument ~top_y ~offset_x
    ~(notes : Fretboard_position.t list option) =
  let string_count = Instrument.string_count instrument in
  let note_count = Option.value_map notes ~default:2 ~f:List.length in
  let layout = layout config ~canvas_width ~string_count ~note_count in
  draw_string_lines config layout ~canvas_width ~instrument ~top_y ~offset_x;
  Option.iter notes ~f:(draw_notes config layout ~top_y ~offset_x);
  let rule_y =
    top_y
    + to_pixels
        (Tab_layout.canvas_height ~margin:config.margin ~string_count
           ~row_spacing:config.row_spacing)
    + to_pixels config.rule_gap
  in
  draw_rule config ~canvas_width ~y:rule_y ~offset_x;
  rule_y + to_pixels config.rule_thickness + to_pixels config.rule_gap
;;
