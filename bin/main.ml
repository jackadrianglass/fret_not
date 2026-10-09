open! Base
open Fret_not

let () =
  let fretboard_config = Fretboard_view_config.default in
  let tab_config = Tab_view_config.default in
  let chunk_config = Chunk_view_config.default in
  let string_count = Instrument.string_count fretboard_config.instrument in
  let tab_area_height =
    Tab_layout.canvas_height ~margin:tab_config.margin ~string_count
      ~row_spacing:tab_config.row_spacing
    +. tab_config.rule_gap +. tab_config.rule_thickness +. tab_config.rule_gap
  in
  let page_layout : Page_layout.t =
    { control_bar_height = Float.of_int fretboard_config.control_bar_height
    ; chunk_area_height =
        Chunk_view_config.panel_height chunk_config +. chunk_config.panel_gap
    ; tab_area_height
    ; fretboard_height = Float.of_int fretboard_config.canvas_height
    }
  in
  let min_height = Int.of_float (Page_layout.content_height page_layout) in
  Raylib.init_window fretboard_config.canvas_width min_height
    fretboard_config.window_title;
  (* FLAG_WINDOW_MAXIMIZED is only honored while the window is resizable, so
     both flags land in one call. *)
  Raylib.set_window_state
    Raylib.ConfigFlags.(window_resizable + window_maximized);
  Raylib.set_window_min_size fretboard_config.window_min_width min_height;
  Raylib.set_target_fps fretboard_config.target_fps;
  Fretboard_view.setup fretboard_config;
  Chunk_view.setup ();
  let rec loop (state : Fretboard_view_state.t)
      (chunk_state : Chunk_view_state.t) =
    match Raylib.window_should_close () with
    | true -> Raylib.close_window ()
    | false ->
        let open Raylib in
        (* Recomputed every frame rather than once: window_should_close
           polls window events, so resized dimensions may only become
           current partway through the first few frames. *)
        let window_width = get_screen_width () in
        let window_height = get_screen_height () in
        let frame_fretboard_config : Fretboard_view_config.t =
          { fretboard_config with canvas_width = window_width }
        in
        begin_drawing ();
        clear_background Color.raywhite;
        let content_top_y =
          Int.of_float (Page_layout.content_top_y page_layout)
        in
        let next_chunk_state =
          match Fretboard_view_state.page_of_index state.page_index with
          | Chunk_editing ->
              Chunk_view.draw chunk_config chunk_state ~top_y:content_top_y
                ~offset_x:0
          | Tab_viewing ->
              (* Placeholder until the tab page gets its own data source: an
                 empty tab grid, no notes. *)
              ignore
                (Tab_view.draw tab_config ~canvas_width:window_width
                   ~instrument:fretboard_config.instrument ~top_y:content_top_y
                   ~offset_x:0 ~notes:None);
              chunk_state
        in
        let fretboard_top_y =
          Int.of_float
            (Page_layout.fretboard_top_y page_layout
               ~window_height:(Float.of_int window_height))
        in
        Fretboard_view.draw_fretboard_grid frame_fretboard_config
          ~top_y:fretboard_top_y ~offset_x:0;
        Fretboard_view.draw_fret_positions frame_fretboard_config state
          ~top_y:fretboard_top_y ~offset_x:0;
        let next_state =
          Fretboard_view.draw_controls frame_fretboard_config state ~offset_x:0
            ~offset_y:0
        in
        end_drawing ();
        loop next_state next_chunk_state
  in
  loop Fretboard_view_state.initial Chunk_view_state.initial
;;
