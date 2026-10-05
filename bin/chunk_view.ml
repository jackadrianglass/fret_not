open! Base
open Fret_not

let to_pixels x = Int.of_float x

let layout (config : Chunk_view_config.t) : Chunk_layout.t =
  { start_x = config.margin
  ; cell_width = config.cell_width
  ; cell_height = config.cell_height
  ; cell_gap = config.cell_gap
  }
;;

let degree_options = "1;2;3;4;5;6;7"
let kind_options = "Note;Rest"
let alteration_options = "b;n;#"
let octave_options = "0;1"

let ui_style_color prop =
  Raylib.get_color (Raygui.get_style (Raygui.Control.Default prop))
;;

(* raygui propagates DEFAULT's style properties to every control, so the
   6px Text_padding Fretboard_view.setup installs for its dropdowns also
   shifts toggle and button text right — onto the borders of tight boxes.
   The chunk editor's own controls opt back out. *)
let setup () =
  Raygui.set_style (Raygui.Control.Toggle `Text_padding) 0;
  Raygui.set_style (Raygui.Control.Button `Text_padding) 0
;;

let ui_accent_fill () = ui_style_color `Base_color_pressed
let ui_accent_border () = ui_style_color `Border_color_pressed
let ui_accent_text () = ui_style_color `Text_color_pressed
let ui_text_size () = Raygui.get_style (Raygui.Control.Default `Text_size)
let text_width text = Raylib.measure_text text (ui_text_size ())

let toggle_entry_width (config : Chunk_view_config.t) options =
  String.split options ~on:';'
  |> List.map ~f:(fun entry ->
      Float.of_int (text_width entry) +. config.inspector_control_padding)
  |> List.max_elt ~compare:Float.compare
  |> Option.value ~default:0.
;;

let toggle_group_padding () =
  Float.of_int (Raygui.get_style (Raygui.Control.Toggle `Padding_toggle))
;;

let toggle_width (config : Chunk_view_config.t) options =
  (* GuiToggleGroup draws every entry at the full bounds width, stepping
     right by that width plus the group padding — so the bounds carry one
     entry's width, and the control's footprint is every entry box plus the
     paddings between them. *)
  let count = List.length (String.split options ~on:';') in
  (Float.of_int count *. toggle_entry_width config options)
  +. (Float.of_int (count - 1) *. toggle_group_padding ())
;;

let dropdown_width (config : Chunk_view_config.t) options =
  (* Dropdown text is centered but shifted right by DEFAULT's Text_padding
     (GetTextBounds insets x without shrinking the width), so the width has
     to budget that padding alongside the text, the arrow, and the gap. *)
  let widest =
    String.split options ~on:';'
    |> List.map ~f:text_width
    |> List.max_elt ~compare:Int.compare
    |> Option.value ~default:0
  in
  Float.of_int widest +. config.inspector_control_padding
  +. config.dropdown_arrow_width
  +. Float.of_int (Raygui.get_style (Raygui.Control.Default `Text_padding))
;;

let alteration_index = function
  | Alteration.Flat | Alteration.Double_flat -> 0
  | Alteration.Natural -> 1
  | Alteration.Sharp | Alteration.Double_sharp -> 2
;;

let alteration_of_index = function
  | 0 -> Alteration.Flat
  | 1 -> Alteration.Natural
  | _ -> Alteration.Sharp
;;

let octave_index octave = if octave > 0 then 1 else 0

let selected_note_value state ~default ~f =
  match Chunk_view_state.selected_slot state with
  | Some (Chunk.Note (dr : Degree_reference.t)) -> f dr
  | _ -> default
;;

let kind_index state =
  match Chunk_view_state.selected_slot state with
  | Some Chunk.Rest -> 1
  | _ -> 0
;;

let handle_strip_click (config : Chunk_view_config.t)
    (state : Chunk_view_state.t) ~top_y ~offset_x =
  let open Raylib in
  if is_mouse_button_pressed MouseButton.Left then
    let x = Float.of_int (get_mouse_x () - offset_x) in
    let y = Float.of_int (get_mouse_y () - top_y) in
    let slot_count = List.length (Chunk_view_state.chunk state) in
    Chunk_layout.slot_at_point (layout config) ~slot_count ~x ~y
    |> Option.value_map ~default:state ~f:(fun slot_index ->
        Chunk_view_state.select_slot state ~slot_index)
  else state
;;

let draw_centered_text ~text ~center_x ~center_y ~font_size ~color =
  let open Raylib in
  let width = measure_text text font_size in
  draw_text text
    (center_x - (width / 2))
    (center_y - (font_size / 2))
    font_size color
;;

let draw_cell (config : Chunk_view_config.t) ~label ~rest ~selected ~x ~y =
  let open Raylib in
  let width = to_pixels config.cell_width in
  let height = to_pixels config.cell_height in
  let fill = if rest then Color.lightgray else ui_accent_fill () in
  let text_color = if rest then Color.darkgray else ui_accent_text () in
  draw_rectangle x y width height fill;
  draw_rectangle_lines x y width height (ui_accent_border ());
  (if selected then
     let extra = config.cursor_outline_extra in
     draw_rectangle_lines_ex
       (Rectangle.create
          (Float.of_int x -. extra)
          (Float.of_int y -. extra)
          (Float.of_int width +. (2. *. extra))
          (Float.of_int height +. (2. *. extra)))
       extra Color.black);
  draw_centered_text ~text:label
    ~center_x:(x + (width / 2))
    ~center_y:(y + (height / 2))
    ~font_size:config.slot_font_size ~color:text_color
;;

let draw_strip (config : Chunk_view_config.t) (state : Chunk_view_state.t)
    ~top_y ~offset_x : Chunk_view_state.t =
  let open Raylib in
  let chunk_layout = layout config in
  let slot_count = List.length (Chunk_view_state.chunk state) in
  List.iteri (Chunk_view_state.chunk state) ~f:(fun slot_index slot ->
      let x =
        offset_x + to_pixels (Chunk_layout.cell_x chunk_layout ~slot_index)
      in
      draw_cell config
        ~label:(Chunk_view_state.slot_label slot)
        ~rest:(match slot with Chunk.Rest -> true | _ -> false)
        ~selected:(Int.equal slot_index state.cursor_index)
        ~x ~y:top_y);
  let add_x =
    Float.of_int offset_x
    +. Chunk_layout.cell_x chunk_layout ~slot_index:slot_count
  in
  let add_clicked =
    Raygui.button
      (Rectangle.create add_x (Float.of_int top_y) config.cell_width
         config.cell_height)
      "+"
  in
  if add_clicked && slot_count < config.max_slots then
    Chunk_view_state.add_slot state ~slot:Chunk.Rest
  else state
;;

let draw_inspector (config : Chunk_view_config.t) (state : Chunk_view_state.t)
    ~y ~offset_x =
  let open Raylib in
  let kind_width = toggle_width config kind_options in
  let kind_entry_width = toggle_entry_width config kind_options in
  let degree_width = dropdown_width config degree_options in
  let alteration_width = toggle_width config alteration_options in
  let alteration_entry_width = toggle_entry_width config alteration_options in
  let octave_width = toggle_width config octave_options in
  let octave_entry_width = toggle_entry_width config octave_options in
  let xs =
    Row_layout.x_positions
      ~start_x:(Float.of_int offset_x +. config.margin)
      ~gap:config.inspector_gap
      [ kind_width
      ; degree_width
      ; alteration_width
      ; octave_width
      ; config.button_width
      ]
  in
  let height = config.inspector_control_height in
  let kind_x = List.nth_exn xs 0
  and degree_x = List.nth_exn xs 1
  and alteration_x = List.nth_exn xs 2
  and octave_x = List.nth_exn xs 3
  and remove_x = List.nth_exn xs 4 in
  let state =
    let kind = kind_index state in
    let next =
      Raygui.toggle_group
        (Rectangle.create kind_x y kind_entry_width height)
        kind_options kind
    in
    if Int.equal next kind then state
    else Chunk_view_state.set_selected_rest state ~rest:(Int.equal next 1)
  in
  let degree_index =
    selected_note_value state ~default:0 ~f:(fun (dr : Degree_reference.t) ->
        Degree_reference.degree dr - 1)
  in
  let degree, degree_toggled =
    Raygui.dropdown_box
      (Rectangle.create degree_x y degree_width height)
      degree_options degree_index state.degree_dropdown_open
  in
  let degree_dropdown_open =
    if degree_toggled then not state.degree_dropdown_open
    else state.degree_dropdown_open
  in
  let state =
    if Int.equal degree degree_index then state
    else Chunk_view_state.set_selected_degree state ~degree:(degree + 1)
  in
  let state =
    let alteration =
      selected_note_value state ~default:(alteration_index Alteration.Natural)
        ~f:(fun (dr : Degree_reference.t) ->
          alteration_index (Degree_reference.alteration dr))
    in
    let next =
      Raygui.toggle_group
        (Rectangle.create alteration_x y alteration_entry_width height)
        alteration_options alteration
    in
    if Int.equal next alteration then state
    else
      Chunk_view_state.set_selected_alteration state
        ~alteration:(alteration_of_index next)
  in
  let state =
    let octave =
      selected_note_value state ~default:0 ~f:(fun (dr : Degree_reference.t) ->
          octave_index (Degree_reference.octave dr))
    in
    let next =
      Raygui.toggle_group
        (Rectangle.create octave_x y octave_entry_width height)
        octave_options octave
    in
    if Int.equal next octave then state
    else Chunk_view_state.set_selected_octave state ~octave:next
  in
  let state =
    if
      Raygui.button (Rectangle.create remove_x y config.button_width height) "-"
    then Chunk_view_state.remove_selected state
    else state
  in
  { state with degree_dropdown_open }
;;

let draw (config : Chunk_view_config.t) (state : Chunk_view_state.t) ~top_y
    ~offset_x : Chunk_view_state.t =
  let state = handle_strip_click config state ~top_y ~offset_x in
  let state = draw_strip config state ~top_y ~offset_x in
  let inspector_y =
    Float.of_int
      (top_y + to_pixels config.cell_height + to_pixels config.strip_gap)
  in
  draw_inspector config state ~y:inspector_y ~offset_x
;;
