open! Base

(* Compile-only probe of every widget/API pattern destined for the UI examples
   reference. Not linked or run; exists so the notes' snippets are
   type-checked against the vendored bindings. *)

let rect x y w h = Raylib.Rectangle.create x y w h

let v2 x y = Raylib.Vector2.create x y

type state = {
  scroll : Raylib.Vector2.t;
  spinner_val : int;
  spinner_edit : bool;
  text : string;
  text_edit : bool;
  multi_text : string;
  multi_edit : bool;
  list_active : int;
  list_scroll : int;
  list_ex_active : int;
  list_ex_focus : int;
  list_ex_scroll : int;
  dropdown_active : int;
  dropdown_open : bool;
  toggle_active : bool;
  toggle_group_active : int;
  combo_active : int;
  checked : bool;
  slider_val : float;
  slider_bar_val : float;
  progress_val : float;
  color_val : Raylib.Color.t;
  hue_val : float;
  alpha_val : float;
  show_modal : bool;
  modal_text : string;
  camera : Raylib.Camera2D.t;
  font : Raylib.Font.t;
  ui_layer : Raylib.RenderTexture.t;
}

let setup () =
  Raylib.init_window 800 450 "probe";
  Raylib.set_target_fps 60;
  {
    scroll = v2 0. 0.;
    spinner_val = 0;
    spinner_edit = false;
    text = "Text box";
    text_edit = false;
    multi_text = "multi";
    multi_edit = false;
    list_active = 0;
    list_scroll = 0;
    list_ex_active = 0;
    list_ex_focus = 0;
    list_ex_scroll = 0;
    dropdown_active = 0;
    dropdown_open = false;
    toggle_active = false;
    toggle_group_active = 0;
    combo_active = 0;
    checked = false;
    slider_val = 0.;
    slider_bar_val = 50.;
    progress_val = 0.5;
    color_val = Raylib.Color.raywhite;
    hue_val = 0.;
    alpha_val = 1.;
    show_modal = false;
    modal_text = "";
    camera =
      Raylib.Camera2D.create (v2 0. 0.) (v2 0. 0.) 0. 1.;
    font = Raylib.load_font "custom_font.ttf";
    ui_layer = Raylib.load_render_texture 800 450;
  }

let draw_panned_fretboard s =
  (* camera pan + zoom-to-cursor: the pattern for a pannable fretboard view *)
  let wheel = Raylib.get_mouse_wheel_move () in
  if not (Float.equal wheel 0.) then (
    let mouse = Raylib.get_mouse_position () in
    let world = Raylib.get_screen_to_world_2d mouse s.camera in
    Raylib.Camera2D.set_offset s.camera mouse;
    Raylib.Camera2D.set_target s.camera world;
    let factor = 1. +. (0.25 *. Float.abs wheel) in
    let factor = if Float.(wheel < 0.) then 1. /. factor else factor in
    let z = Raylib.Camera2D.zoom s.camera *. factor in
    Raylib.Camera2D.set_zoom s.camera (Float.max 0.125 (Float.min 64. z)));
  if Raylib.is_mouse_button_down Raylib.MouseButton.Right then (
    let delta = Raylib.get_mouse_delta () in
    let inv = -1. /. Raylib.Camera2D.zoom s.camera in
    Raylib.Camera2D.set_target s.camera
      (Raylib.Vector2.add (Raylib.Camera2D.target s.camera)
         (Raylib.Vector2.add_value delta inv)));
  Raylib.begin_mode_2d s.camera;
  Raylib.draw_circle 400 225 50. Raylib.Color.maroon;
  Raylib.end_mode_2d ()

let draw_cached_layer s =
  (* render-texture caching: draw static content once, blit per frame *)
  Raylib.begin_texture_mode s.ui_layer;
  Raylib.clear_background Raylib.Color.blank;
  Raylib.draw_text "cached layer" 10 10 20 Raylib.Color.gray;
  Raylib.end_texture_mode ();
  let tex = Raylib.RenderTexture.texture s.ui_layer in
  (* render textures are stored bottom-up: source rect must flip y *)
  let tw = Float.of_int (Raylib.Texture.width tex) in
  let th = Float.of_int (Raylib.Texture.height tex) in
  let src = rect 0. th tw (Float.neg th) in
  Raylib.draw_texture_rec tex src (v2 0. 0.) Raylib.Color.white

let draw_scroll_panel s =
  (* the addr pattern: create the Vector2 once, pass its address every frame,
     read the mutated offset back through the accessors *)
  let bounds = rect 20. 40. 200. 150. in
  let content = rect 0. 0. 340. 340. in
  let view = Raygui.scroll_panel bounds content (Raylib.addr s.scroll) in
  let sx = Raylib.Vector2.x s.scroll in
  let sy = Raylib.Vector2.y s.scroll in
  Raylib.begin_scissor_mode
    (Int.of_float (Raylib.Rectangle.x view))
    (Int.of_float (Raylib.Rectangle.y view))
    (Int.of_float (Raylib.Rectangle.width view))
    (Int.of_float (Raylib.Rectangle.height view));
  Raylib.draw_rectangle
    (Int.of_float (Raylib.Rectangle.x bounds +. sx))
    (Int.of_float (Raylib.Rectangle.y bounds +. sy))
    340 340 (Raylib.fade Raylib.Color.red 0.1);
  let mouse_cell =
    Raygui.grid
      (rect
         (Raylib.Rectangle.x bounds +. sx)
         (Raylib.Rectangle.y bounds +. sy)
         340. 340.)
      16. 3
  in
  ignore (Raylib.Vector2.x mouse_cell);
  Raylib.end_scissor_mode ()

let draw_text_widgets s =
  Raygui.set_style (Raygui.Control.TextBox `Text_alignment)
    (Raygui.TextAlignment.to_int Raygui.TextAlignment.Center);
  let spinner_val, spinner_edit =
    match
      Raygui.spinner (rect 25. 135. 125. 30.) "" s.spinner_val ~min:0 ~max:100
        s.spinner_edit
    with
    | v, true -> (v, not s.spinner_edit)
    | v, false -> (v, s.spinner_edit)
  in
  let text, text_edit =
    match Raygui.text_box (rect 25. 215. 125. 30.) s.text s.text_edit with
    | v, true -> (v, not s.text_edit)
    | v, false -> (v, s.text_edit)
  in
  let multi_text, multi_edit =
    match
      Raygui.text_box_multi (rect 320. 25. 225. 140.) s.multi_text s.multi_edit
    with
    | v, true -> (v, not s.multi_edit)
    | v, false -> (v, s.multi_edit)
  in
  Raygui.set_style (Raygui.Control.TextBox `Text_alignment)
    (Raygui.TextAlignment.to_int Raygui.TextAlignment.Left);
  (spinner_val, spinner_edit, text, text_edit, multi_text, multi_edit)

let draw_lists s =
  let list_active, list_scroll =
    Raygui.list_view (rect 165. 25. 140. 140.) "a;b;c;d;e;f" s.list_scroll
      s.list_active
  in
  let list_ex_active, list_ex_focus, list_ex_scroll =
    Raygui.list_view_ex (rect 165. 180. 140. 200.)
      [ "This"; "is"; "list_view_ex"; "with"; "items" ]
      s.list_ex_focus s.list_ex_scroll s.list_ex_active
  in
  (list_active, list_scroll, list_ex_active, list_ex_focus, list_ex_scroll)

let draw_basics s =
  let checked = Raygui.check_box (rect 25. 108. 15. 15.) "FORCE" s.checked in
  let combo_active =
    Raygui.combo_box (rect 25. 470. 125. 30.) "ONE;TWO;THREE" s.combo_active
  in
  let dropdown_active, dropdown_open =
    match
      Raygui.dropdown_box (rect 25. 25. 125. 30.) "ONE;TWO;THREE"
        s.dropdown_active s.dropdown_open
    with
    | v, true -> (v, not s.dropdown_open)
    | v, false -> (v, s.dropdown_open)
  in
  let toggle_active =
    Raygui.toggle (rect 165. 25. 140. 25.) "toggle me" s.toggle_active
  in
  let toggle_group_active =
    Raygui.toggle_group (rect 165. 400. 140. 25.) "ONE;TWO;THREE"
      s.toggle_group_active
  in
  let slider_val =
    Raygui.slider (rect 355. 400. 165. 20.) "TEST"
      (Printf.sprintf "%2.2f" s.slider_val)
      s.slider_val ~min:(-50.) ~max:100.
  in
  let slider_bar_val =
    Raygui.slider_bar (rect 320. 430. 200. 20.) ""
      (Printf.sprintf "%i" (Int.of_float s.slider_bar_val))
      s.slider_bar_val ~min:0. ~max:100.
  in
  let progress_val =
    Raygui.progress_bar (rect 320. 460. 200. 20.) "" "" s.progress_val
      ~min:0. ~max:1.
  in
  let color_val = Raygui.color_picker (rect 320. 185. 196. 192.) s.color_val in
  let hue_val = Raygui.color_bar_hue (rect 520. 185. 20. 192.) s.hue_val in
  let alpha_val = Raygui.color_bar_alpha (rect 520. 400. 100. 20.) s.alpha_val in
  ( checked,
    combo_active,
    dropdown_active,
    dropdown_open,
    toggle_active,
    toggle_group_active,
    slider_val,
    slider_bar_val,
    progress_val,
    color_val,
    hue_val,
    alpha_val )

let draw_containers () =
  Raygui.panel (rect 600. 25. 180. 200.);
  Raygui.group_box (rect 600. 240. 180. 150.) "STATES";
  Raygui.line (rect 25. 510. 700. 10.) "divider";
  Raygui.dummy_rec (rect 25. 525. 125. 20.) "placeholder";
  ignore (Raygui.window_box (rect 610. 30. 160. 90.) "Window Box");
  Raygui.status_bar (rect 25. 550. 700. 25.) "status message"

let draw_states () =
  Raygui.lock ();
  Raygui.set_state Raygui.ControlState.Pressed;
  ignore (Raygui.button (rect 610. 130. 160. 30.) "#15#PRESSED");
  Raygui.set_state Raygui.ControlState.Disabled;
  ignore (Raygui.button (rect 610. 165. 160. 30.) "DISABLED");
  Raygui.set_state Raygui.ControlState.Normal;
  Raygui.enable ();
  Raygui.disable ();
  Raygui.unlock ();
  Raygui.fade 0.8;
  Raygui.load_style_default ();
  Raygui.load_style "theme.rgs"

let draw_modal s =
  if s.show_modal then (
    Raygui.lock ();
    Raylib.draw_rectangle 0 0 (Raylib.get_screen_width ())
      (Raylib.get_screen_height ())
      (Raylib.fade Raylib.Color.raywhite 0.8);
    let text, res =
      Raygui.text_input_box
        (rect
           ((Float.of_int (Raylib.get_screen_width ()) /. 2.) -. 120.)
           ((Float.of_int (Raylib.get_screen_height ()) /. 2.) -. 60.)
           240. 140.)
        "Save file as..." "Introduce a name" "Ok;Cancel" s.modal_text
    in
    if res = 0 || res = 1 then ignore text)
  else if Raygui.button (rect 25. 255. 125. 30.) "Save File" then ()

let draw_fonts s =
  let size = Printf.sprintf "mouse: %d, %d" (Raylib.get_mouse_x ())
      (Raylib.get_mouse_y ())
  in
  Raylib.draw_text size 10 10 20 Raylib.Color.darkgray;
  let measured = Raylib.measure_text_ex s.font size 20. 1. in
  Raylib.draw_text_ex s.font size
    (v2 (Raylib.Vector2.x measured) 40.)
    20. 1. Raylib.Color.maroon

let draw_hit_test () =
  let mouse = Raylib.get_mouse_position () in
  let hovered =
    Raylib.check_collision_point_rec mouse (rect 10. 10. 100. 40.)
  in
  if hovered then Raylib.set_mouse_cursor Raylib.MouseCursor.Ibeam
  else Raylib.set_mouse_cursor Raylib.MouseCursor.Default

let rec loop s =
  if Raylib.window_should_close () then Raylib.close_window ()
  else
    let open Raylib in
    begin_drawing ();
    clear_background Color.raywhite;
    draw_panned_fretboard s;
    draw_cached_layer s;
    draw_scroll_panel s;
    let spinner_val, spinner_edit, text, text_edit, multi_text, multi_edit =
      draw_text_widgets s
    in
    let list_active, list_scroll, list_ex_active, list_ex_focus, list_ex_scroll
        =
      draw_lists s
    in
    let ( checked,
          combo_active,
          dropdown_active,
          dropdown_open,
          toggle_active,
          toggle_group_active,
          slider_val,
          slider_bar_val,
          progress_val,
          color_val,
          hue_val,
          alpha_val ) =
      draw_basics s
    in
    draw_containers ();
    draw_states ();
    draw_modal s;
    draw_fonts s;
    draw_hit_test ();
    end_drawing ();
    loop
      {
        s with
        spinner_val;
        spinner_edit;
        text;
        text_edit;
        multi_text;
        multi_edit;
        list_active;
        list_scroll;
        list_ex_active;
        list_ex_focus;
        list_ex_scroll;
        dropdown_active;
        dropdown_open;
        toggle_active;
        toggle_group_active;
        combo_active;
        checked;
        slider_val;
        slider_bar_val;
        progress_val;
        color_val;
        hue_val;
        alpha_val;
      }

let () = setup () |> loop
