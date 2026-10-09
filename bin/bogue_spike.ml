open! Base
open Bogue
open Tsdl_ttf
module W = Widget
module L = Layout

type state =
  { tonic : int
  ; quality : int
  ; hollow : bool
  ; wide_tonics : bool
  }

let full_tonics =
  [| "C"; "G"; "D"; "A"; "E"; "B"; "F#"; "C#"; "F"; "Bb"; "Eb"; "Ab" |]
;;

let short_tonics = [| "C"; "G"; "D"; "A"; "E" |]
let qualities = [| "Major"; "Minor" |]
let state = ref { tonic = 0; quality = 0; hollow = false; wide_tonics = true }
let wrap fret = Int.rem (fret - 1) 12 + 1

(* mock triad: root twice (two octaves), third and fifth on the middle strings *)
let dots st =
  let root = st.tonic + 1 in
  let third = root + if st.quality = 0 then 4 else 3 in
  [ (1, root, true)
  ; (2, wrap third, false)
  ; (3, wrap (root + 7), false)
  ; (4, root, true)
  ]
;;

let n_strings = 6
let n_frets = 12

(* high string on top, like a fretboard diagram *)
let strings = [| "E4"; "B3"; "G3"; "D3"; "A2"; "E2" |]
let color name = Draw.opaque (Draw.find_color name)

(* Bogue draws widget text but not canvas text; SDL_ttf is usable from
   inside the render queue, which runs after SDL init *)
let font_ref : Ttf.font option ref = ref None

let log msg = Stdlib.prerr_endline ("spike: " ^ msg)

let font () =
  match !font_ref with
  | Some f -> f
  | None ->
      let path =
        match Theme.get_font_path_opt "Ubuntu-R.ttf" with
        | Some p -> p
        | None -> "/System/Library/Fonts/Supplemental/Arial.ttf"
      in
      if not (Ttf.was_init ()) then ignore (Ttf.init ());
      log ("opening font " ^ path);
      let f =
        match Ttf.open_font path (Theme.scale_int 14) with
        | Ok f -> f
        | Error (`Msg m) ->
            log ("open_font failed: " ^ m);
            failwith "spike: cannot open font"
      in
      font_ref := Some f;
      f
;;

let text_size f text =
  match Ttf.size_utf8 f text with Ok (w, h) -> (w, h) | Error _ -> (0, 0)
;;

let draw_text renderer text (r, g, b) x y =
  let open Tsdl in
  let fg = Sdl.Color.create ~r ~g ~b ~a:255 in
  match Ttf.render_utf8_blended (font ()) text fg with
  | Error (`Msg m) -> log ("render_utf8: " ^ m)
  | Ok surface -> (
      match Sdl.create_texture_from_surface renderer surface with
      | Error (`Msg m) ->
          Sdl.free_surface surface;
          log ("texture_from_surface: " ^ m)
      | Ok tex ->
          (* textures from alpha surfaces do not blend by default *)
          (match Sdl.set_texture_blend_mode tex Sdl.Blend.mode_blend with
          | Ok () -> ()
          | Error (`Msg m) -> log ("blend mode: " ^ m));
          let w, h =
            match Sdl.query_texture tex with
            | Ok (_, _, (w, h)) -> (w, h)
            | Error (`Msg m) ->
                log ("query_texture: " ^ m);
                (0, 0)
          in
          let dst = Sdl.Rect.create ~x ~y ~w ~h in
          (match Sdl.render_copy renderer tex ~dst with
          | Ok () -> ()
          | Error (`Msg m) -> log ("render_copy: " ^ m));
          Sdl.destroy_texture tex;
          Sdl.free_surface surface)
;;

let draw_scene area st =
  let wood = color "bisque" in
  let string_col = color "gainsboro" in
  let root_col = color "indianred" in
  let dot_col = color "white" in
  Sdl_area.clear area;
  (* all geometry derives from drawing_size inside the command, so a window
     resize re-renders correctly (plus the on_resize hook below) *)
  Sdl_area.add area (fun renderer ->
      let w, h = Sdl_area.drawing_size area in
      Sdl_area.fill_rectangle area ~color:(color "midnightblue") ~w ~h (0, 0);
      let left = w / 10 in
      let right = w - (w / 20) in
      let top = h / 8 in
      let bottom = h - (h / 8) in
      let fret_x f = left + (f * (right - left) / n_frets) in
      let string_y s = top + (s * (bottom - top) / (n_strings - 1)) in
      Sdl_area.fill_rectangle area ~color:wood
        ~w:(fret_x n_frets - fret_x 0)
        ~h:(bottom - top + 20)
        (fret_x 0, top - 10);
      for f = 0 to n_frets do
        Sdl_area.draw_line area
          ~color:(if f = 0 then color "black" else color "gray")
          ~thick:(if f = 0 then 8 else 2)
          (fret_x f, top - 10)
          (fret_x f, bottom + 10)
      done;
      for s = 0 to n_strings - 1 do
        Sdl_area.draw_line area ~color:string_col
          ~thick:(1 + (s / 2))
          (fret_x 0, string_y s)
          (fret_x n_frets, string_y s)
      done;
      let radius = (bottom - top) / 18 in
      List.iter
        ~f:(fun (s, f, is_root) ->
          let col = if is_root then root_col else dot_col in
          let pos = ((fret_x (f - 1) + fret_x f) / 2, string_y s) in
          if st.hollow then
            Sdl_area.draw_circle area ~color:col ~thick:3 ~radius pos
          else Sdl_area.fill_circle area ~color:col ~radius pos)
        (dots st);
      (* canvas text: open-string labels and fret numbers *)
      let f = font () in
      for s = 0 to n_strings - 1 do
        let tw, th = text_size f strings.(s) in
        draw_text renderer strings.(s) (20, 20, 20)
          (left - tw - 12)
          (string_y s - (th / 2))
      done;
      for fret = 1 to n_frets do
        let label = Int.to_string fret in
        let tw, _ = text_size f label in
        draw_text renderer label (20, 20, 20)
          (((fret_x (fret - 1) + fret_x fret) / 2) - (tw / 2))
          (bottom + 12)
      done);
  Sdl_area.update area
;;

let () =
  let area_w = W.sdl_area ~w:880 ~h:340 () in
  let area = W.get_sdl_area area_w in
  let refresh () = draw_scene area !state in
  let tonic_options () =
    if !state.wide_tonics then full_tonics else short_tonics
  in
  let make_tonic_sel () =
    Select.create
      ~action:(fun i ->
        state := { !state with tonic = i };
        refresh ())
      (tonic_options ())
      (Int.min !state.tonic (Array.length (tonic_options ()) - 1))
  in
  let tonic_sel = ref (make_tonic_sel ()) in
  (* the position dropdown in the real app derives its options from the
     current shape list; rebuilding a Select via replace_room is the way
     to change its options *)
  let rebuild_tonic_sel () =
    let next = make_tonic_sel () in
    ignore (L.replace_room !tonic_sel ~by:next);
    tonic_sel := next
  in
  let quality_sel =
    Select.create
      ~action:(fun i ->
        state := { !state with quality = i };
        refresh ())
      qualities !state.quality
  in
  let hollow_btn =
    W.button ~kind:Button.Switch
      ~action:(fun on ->
        state := { !state with hollow = on };
        refresh ())
      "Hollow dots"
  in
  let rebuild_btn =
    W.button
      ~action:(fun _ ->
        state :=
          { !state with
            wide_tonics = not !state.wide_tonics
          ; tonic = Int.min !state.tonic (Array.length (tonic_options ()) - 1)
          };
        rebuild_tonic_sel ();
        refresh ())
      "Rebuild tonics"
  in
  let quit_btn = W.button ~action:(fun _ -> Bogue.quit ()) "Quit" in
  refresh ();
  let controls =
    L.flat ~sep:16 ~margins:10
      [ !tonic_sel
      ; quality_sel
      ; L.resident hollow_btn
      ; L.resident rebuild_btn
      ; L.resident quit_btn
      ]
  in
  let canvas_room = L.resident area_w in
  L.on_resize canvas_room (fun () -> Sdl_area.update area);
  let board = L.tower ~margins:8 [ controls; canvas_room ] in
  let window = Window.create board in
  Sync.push (fun () -> Window.maximize_width window);
  let gui = Main.of_windows [ window ] in
  Main.run gui;
  Bogue.quit ()
;;
