open! Base
open Fret_not

let () =
  let fretboard_config = Fretboard_view_config.default in
  let tab_config = Tab_view_config.default in
  let chunk_config = Chunk_view_config.default in
  let chunk_area_height =
    Int.of_float
      (Chunk_view_config.panel_height chunk_config +. chunk_config.panel_gap)
  in
  let tab_area_height =
    Int.of_float
      (Tab_layout.canvas_height ~margin:tab_config.margin
         ~string_count:(Instrument.string_count fretboard_config.instrument)
         ~row_spacing:tab_config.row_spacing)
  in
  let content_height =
    fretboard_config.control_bar_height + chunk_area_height + tab_area_height
    + Int.of_float (tab_config.rule_gap *. 2.)
    + fretboard_config.canvas_height
  in
  Raylib.init_window fretboard_config.canvas_width content_height
    fretboard_config.window_title;
  Raylib.set_target_fps fretboard_config.target_fps;
  Fretboard_view.setup fretboard_config;
  Chunk_view.setup ();
  (* Resizes the window to match the monitor resolution itself, in the same
     points-based coordinate system get_screen_width/height report in - no
     manual monitor-pixel math (which mismatches on a HiDPI display, where
     get_monitor_width/height report physical pixels instead). *)
  Raylib.toggle_borderless_windowed ();
  let rec loop (state : Fretboard_view_state.t)
      (chunk_state : Chunk_view_state.t) =
    match Raylib.window_should_close () with
    | true -> Raylib.close_window ()
    | false ->
        let open Raylib in
        (* Recomputed every frame rather than once: window_should_close
           polls window events, so the resized dimensions may only become
           current partway through the first few frames. *)
        let screen_width = get_screen_width () in
        let screen_height = get_screen_height () in
        (* Width fills the screen edge to edge (offset_x always 0); height
           stays at its own fixed size and is centered vertically, same as
           before. *)
        let content_width = screen_width in
        let offset_x = 0 in
        let offset_y = Int.max 0 ((screen_height - content_height) / 2) in
        let frame_fretboard_config : Fretboard_view_config.t =
          { fretboard_config with canvas_width = content_width }
        in
        begin_drawing ();
        clear_background Color.raywhite;
        let notes =
          Fretboard_view_state.selected_position state ~config:fretboard_config
        in
        let editor_top_y = offset_y + fretboard_config.control_bar_height in
        let next_chunk_state =
          Chunk_view.draw chunk_config chunk_state ~top_y:editor_top_y ~offset_x
        in
        let fretboard_top_y =
          Tab_view.draw tab_config ~canvas_width:content_width
            ~instrument:fretboard_config.instrument
            ~top_y:(editor_top_y + chunk_area_height)
            ~offset_x ~notes
        in
        Fretboard_view.draw_fretboard_grid frame_fretboard_config
          ~top_y:fretboard_top_y ~offset_x;
        Fretboard_view.draw_fret_positions frame_fretboard_config state
          ~top_y:fretboard_top_y ~offset_x;
        let next_state =
          Fretboard_view.draw_controls frame_fretboard_config state ~offset_x
            ~offset_y
        in
        end_drawing ();
        loop next_state next_chunk_state
  in
  loop Fretboard_view_state.initial Chunk_view_state.initial
;;
