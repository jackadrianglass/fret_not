# raylib / raygui UI examples cookbook (OCaml bindings)

Recipes for building UIs with the vendored raylib/raygui 2.2.2 OCaml bindings,
aimed at fret_not's future screens (chunk library, practice flow, pannable
fretboard). Companion to `raylib-raygui-notes.md` (which holds the quirks);
this file holds working patterns. Every snippet below was compile-checked
against the exact vendored bindings in `dune.lock` via
`scratch/raygui_ui_probe.ml` (see "Re-verifying" at the bottom).

## Where examples live

Nothing is vendored locally — the binding package ships only sources:

- **OCaml examples**: github.com/tjammer/raylib-ocaml/tree/main/examples —
  `gui/controls_test_suite.ml` is a complete OCaml walkthrough of every
  raygui widget (the source of most idioms here). Also `core/core_2d_camera*.ml`
  (pan/zoom), `core/core_input_mouse_wheel.ml`, `text/text_input_box.ml`
  (raylib-side text input), `textures/textures_srcrec_dstrec.ml`.
  CAUTION: examples on repo `main` target a newer binding than our 2.2.2 —
  they use bare `Raygui.TextBox` constructors, which are
  `Raygui.Control.TextBox` in our version.
- **raygui C examples**: github.com/raysan5/raygui/tree/master/examples —
  `controls_test_suite/`, `scroll_panel/`, `custom_file_dialog`,
  `textbox_extended`. `scroll_panel.c` is the scissor-mode reference.
- **raygui C source (authoritative)**:
  `_build/_private/default/.pkg/raylib.2.2.2-*/source/src/c/vendor/raygui/src/raygui.h`
  — read a widget's C before trusting its behavior.
- **Binding interface**: `raylib-ocaml-docs.txt` (repo root) for raylib;
  raygui's signatures live in the vendored `raygui.mli`
  (`_build/_private/default/.pkg/raygui.2.2.2-*/source/src/raygui/raygui.mli`).

## The immediate-mode frame, OCaml shape

Every widget call happens inside `begin_drawing ()/end_drawing ()` each frame;
state is threaded through the recursive loop, never mutated in place (except
ctypes structs — see scroll panel below):

```ocaml
let rec loop s =
  if Raylib.window_should_close () then Raylib.close_window ()
  else
    let open Raylib in
    begin_drawing ();
    clear_background Color.raywhite;
    let checked = Raygui.check_box bounds "FORCE" s.checked in
    end_drawing ();
    loop { s with checked }
```

House style note: our `bin/main.ml` already follows this; keep geometry in
`lib/layout` (floats) and convert at the raylib boundary.

### The edit-mode idiom

`dropdown_box`, `spinner`, `value_box`, `text_box`, `text_box_multi` return a
toggle pulse in their last component — the same trick as the dropdown XOR
already documented in `raylib-raygui-notes.md`, generalized:

```ocaml
let spinner_val, spinner_edit =
  match
    Raygui.spinner bounds "" s.spinner_val ~min:0 ~max:100 s.spinner_edit
  with
  | v, true -> (v, not s.spinner_edit)   (* edit mode entered/left this frame *)
  | v, false -> (v, s.spinner_edit)      (* value may still have changed *)
in
```

## Widget inventory (binding signatures + notes)

All bounds are `Raylib.Rectangle.t` (floats). Semicolon-separated option
strings; icon specs `#NNN#` inline (icons: 256 embedded 16x16 glyphs,
editable via the rGuiIcons tool; `GuiIconText`/`GuiDrawIcon` are NOT bound —
icons are reachable only through `#NNN#` label prefixes).

```ocaml
(* containers *)
Raygui.window_box        : Rectangle.t -> string -> bool          (* close X clicked *)
Raygui.group_box         : Rectangle.t -> string -> unit
Raygui.panel             : Rectangle.t -> unit
Raygui.line              : Rectangle.t -> string -> unit           (* separator, w/ optional text *)
Raygui.status_bar        : Rectangle.t -> string -> unit
Raygui.dummy_rec         : Rectangle.t -> string -> unit           (* layout placeholder *)
Raygui.scroll_panel      : Rectangle.t -> Rectangle.t -> Vector2.t Raylib.ptr -> Rectangle.t

(* basics *)
Raygui.label             : Rectangle.t -> string -> unit
Raygui.label_button      : Rectangle.t -> string -> bool
Raygui.button            : Rectangle.t -> string -> bool
Raygui.toggle            : Rectangle.t -> string -> bool -> bool
Raygui.toggle_group      : Rectangle.t -> string -> int -> int
Raygui.check_box         : Rectangle.t -> string -> bool -> bool
Raygui.combo_box         : Rectangle.t -> string -> int -> int      (* stays open while held? no: click cycles *)
Raygui.dropdown_box      : Rectangle.t -> string -> int -> bool -> int * bool
Raygui.slider            : Rectangle.t -> string -> string -> float -> min:float -> max:float -> float
Raygui.slider_bar        : (same shape as slider)
Raygui.progress_bar      : (same shape; display-only)
Raygui.spinner           : Rectangle.t -> string -> int -> min:int -> max:int -> bool -> int * bool
Raygui.value_box         : (same shape as spinner; typed numeric entry)
Raygui.text_box          : Rectangle.t -> string -> bool -> string * bool
Raygui.text_box_multi    : Rectangle.t -> string -> bool -> string * bool   (* multiline *)
Raygui.scroll_bar        : Rectangle.t -> int -> min:int -> max:int -> int  (* standalone scrollbar *)
Raygui.grid              : Rectangle.t -> float -> int -> Vector2.t        (* returns hovered cell *)

(* advanced *)
Raygui.list_view         : Rectangle.t -> string -> int -> int -> int * int        (* (active, scroll) *)
Raygui.list_view_ex      : Rectangle.t -> string list -> int -> int -> int -> int * int * int (* (active, focus, scroll) *)
Raygui.message_box       : Rectangle.t -> string -> string -> string -> int       (* title;msg;buttons -> button id *)
Raygui.text_input_box    : Rectangle.t -> string -> string -> string -> string -> string * int
Raygui.color_picker      : Rectangle.t -> Color.t -> Color.t
Raygui.color_bar_hue     : Rectangle.t -> float -> float
Raygui.color_bar_alpha   : Rectangle.t -> float -> float

(* global state *)
Raygui.lock / unlock / enable / disable : unit -> unit
Raygui.set_state         : ControlState.t -> unit   (* Normal|Focused|Pressed|Disabled — force the look *)
Raygui.fade              : float -> unit            (* global widget alpha *)
Raygui.set_style         : Control.t -> int -> unit
Raygui.get_style         : Control.t -> int
Raygui.set_font          : Raylib.Font.t -> unit     (* all raygui text switches font *)
Raygui.load_style        : string -> unit           (* .rgs theme; load_style_default resets *)
```

Style constructors are `Raygui.Control.(TextBox `Text_alignment)`,
`Raygui.Control.(Default `Background_color)`, etc. — NOT bare `TextBox`
(repo-main examples say otherwise; they're for a newer binding).

## Recipes

### Pannable / zoomable viewport (Camera2D)

The pattern for a fretboard you can pan (right-drag) and zoom under the
cursor (wheel) — from `examples/core/core_2d_camera_mouse_zoom.ml`:

```ocaml
let wheel = Raylib.get_mouse_wheel_move () in
if not (Float.equal wheel 0.) then (
  (* keep the world point under the cursor fixed while zooming *)
  let mouse = Raylib.get_mouse_position () in
  let world = Raylib.get_screen_to_world_2d mouse camera in
  Raylib.Camera2D.set_offset camera mouse;
  Raylib.Camera2D.set_target camera world;
  let factor = 1. +. (0.25 *. Float.abs wheel) in
  let factor = if Float.(wheel < 0.) then 1. /. factor else factor in
  Raylib.Camera2D.set_zoom camera
    (Float.max 0.125 (Float.min 64. (Raylib.Camera2D.zoom camera *. factor))));
if Raylib.is_mouse_button_down Raylib.MouseButton.Right then (
  let delta = Raylib.get_mouse_delta () in
  let inv = -1. /. Raylib.Camera2D.zoom camera in
  Raylib.Camera2D.set_target camera
    (Raylib.Vector2.add (Raylib.Camera2D.target camera)
       (Raylib.Vector2.add_value delta inv)));

Raylib.begin_mode_2d camera;
(* draw the fretboard in world coords here *)
Raylib.end_mode_2d ()
```

`Raylib.Camera2D.create offset target rotation zoom` builds one; `Camera2D.zoom`
is a getter, `set_zoom`/`set_target`/`set_offset` the setters. Remember
`Float.(...)` around every float comparison.

### Scroll panel + scissor mode (a scrollable chunk list)

`Raygui.scroll_panel` is in/out through a `Vector2 ptr`. The binding's ptr is
`Raylib.addr` of a persistent ctypes struct — mutations land in the same
memory, so read the scroll offset back through the ordinary accessors. The
returned rectangle is the visible clip region: feed it to
`begin_scissor_mode` and draw content offset by the scroll vector
(translated from raygui's `scroll_panel.c`):

```ocaml
(* s.scroll : Raylib.Vector2.t lives in the threaded state, created once *)
let bounds = Rectangle.create 20. 40. 200. 150. in
let content = Rectangle.create 0. 0. 340. 340. in   (* full virtual size *)
let view = Raygui.scroll_panel bounds content (Raylib.addr s.scroll) in
Raylib.begin_scissor_mode
  (Int.of_float (Raylib.Rectangle.x view))
  (Int.of_float (Raylib.Rectangle.y view))
  (Int.of_float (Raylib.Rectangle.width view))
  (Int.of_float (Raylib.Rectangle.height view));
(* draw items at bounds.x + scroll.x / bounds.y + scroll.y *)
let hovered_cell =
  Raygui.grid
    (Rectangle.create
       (Raylib.Rectangle.x bounds +. Raylib.Vector2.x s.scroll)
       (Raylib.Rectangle.y bounds +. Raylib.Vector2.y s.scroll)
       340. 340.)
    16. 3
in
Raylib.end_scissor_mode ()
```

For a plain scrollable list (no custom rendering), `list_view` scrolls
itself and returns `(active, scroll_index)` — carry both in state.

### Modal dialog (dim + lock + input box)

From `controls_test_suite.ml`: lock the widgets behind the dialog, dim with
a raylib overlay, and gate everything on the returned button id:

```ocaml
Raygui.lock ();   (* widgets drawn later this frame are grey/disabled *)
Raylib.draw_rectangle 0 0 (Raylib.get_screen_width ()) (Raylib.get_screen_height ())
  (Raylib.fade Raylib.Color.raywhite 0.8);
let text, result =
  Raygui.text_input_box bounds "Save file as..." "Introduce a name"
    "Ok;Cancel" s.modal_text
in
(* result: -1 while open, else the button index (0 = Ok, 1 = Cancel) *)
```

`Raygui.message_box bounds "Title" "Body" "Ok;Cancel"` is the same shape
without a text field. `Raygui.fade 0.8` is an alternative global dim (it
changes the alpha of every widget call after it — restore with `fade 1.`).

### Forcing widget looks (previews, disabled states)

`set_state` overrides the visual state for the next widget call — useful to
draw a control "as if pressed" (e.g. showing the selected fingering) or to
disable a single widget:

```ocaml
Raygui.set_state Raygui.ControlState.Pressed;
ignore (Raygui.button bounds "#15#PRESSED");
Raygui.set_state Raygui.ControlState.Normal
```

`Raygui.enable/disable` do the same globally; always pair and restore.

### Render-texture caching (heavy static content)

Draw the static fretboard grid once into an offscreen texture and blit it
per frame; only the dynamic dots go through the live path. Note the binding
has NO `RenderTexture.width/height` accessors — go through the embedded
texture — and OpenGL render textures are bottom-up, so the source rect must
flip y (negative height):

```ocaml
let tex = Raylib.RenderTexture.texture s.ui_layer in
Raylib.begin_texture_mode s.ui_layer;
Raylib.clear_background Raylib.Color.blank;
(* static drawing *)
Raylib.end_texture_mode ();
let tw = Float.of_int (Raylib.Texture.width tex) in
let th = Float.of_int (Raylib.Texture.height tex) in
Raylib.draw_texture_rec tex (Rectangle.create 0. th tw (Float.neg th))
  (Raylib.Vector2.create 0. 0.) Raylib.Color.white
```

Resizing the window means regenerating the texture (`load_render_texture w h`);
unload the old one (`unload_render_texture`) — leaks are silent.

### Custom fonts (title text, tab glyphs)

```ocaml
let font = Raylib.load_font "path/to.ttf" in
Raygui.set_font font;                    (* widgets now use it *)
let size = Raylib.measure_text_ex font "Some label" 20. 1. in
Raylib.draw_text_ex font "Some label"
  (Raylib.Vector2.create (Raylib.Vector2.x size) 40.) 20. 1. Raylib.Color.maroon
```

raygui's `TEXT_SIZE` style (`` `Text_size `` under `Control.Default`) still
governs widget glyph scaling; measure with `measure_text_ex` at that size
when doing layout.

### Hover hit-testing and cursor feedback

The pattern behind `slot_at_point`-style custom widgets:

```ocaml
let mouse = Raylib.get_mouse_position () in
let hovered = Raylib.check_collision_point_rec mouse bounds in
if hovered then Raylib.set_mouse_cursor Raylib.MouseCursor.Ibeam
else Raylib.set_mouse_cursor Raylib.MouseCursor.Default
```

Mouse wheel for custom scrolling:
`box_y - Int.of_float (Raylib.get_mouse_wheel_move ()) * scroll_speed`.

## Screens these enable in fret_not

- **Chunk library**: `list_view_ex` (chunk names), `text_input_box` (rename),
  `dropdown_box` (filter by degree), `scroll_panel` for a card grid.
- **Practice/pick-and-practice flow** (the unwired `Chunk_solver` path):
  `progress_bar` for position progress, `toggle_group` for string filters,
  `status_bar` for the current chord, `window_box` for the session summary.
- **Fretboard view**: Camera2D pan/zoom for inspecting positions across the
  neck; render-texture cache if per-frame `degrees_in_window` redraw ever
  shows up in profiles.
- **Modal flows**: `message_box` for confirmations, `Raygui.lock` for the
  rest of the screen while a dialog is up.

## Re-verifying

`scratch/raygui_ui_probe.ml` exercises every call above (plus
`value_box`, `combo_box`, `load_style`, `fade`, container widgets,
`get_char_pressed`-free variants). Compile-only; scratch/ is excluded from
the dune project, so drive `ocamlopt` directly against the locked packages:

```sh
R=_build/_private/default/.pkg
OC=$(command -v ocamlopt.opt)
RL=$R/raylib.2.2.2-*/target/lib/raylib
$OC -c -I $RL -I $RL/core -I $RL/audio -I $RL/models -I $RL/rlgl \
  -I $RL/shapes -I $RL/text -I $RL/textures \
  -I $R/raygui.2.2.2-*/target/lib/raygui \
  -I $R/base.v0.17.3-*/target/lib/base \
  -I $R/ctypes.0.24.0-*/target/lib/ctypes \
  scratch/raygui_ui_probe.ml
```

(The `dune rules bin/main.exe` output tells you which of the duplicate
`.pkg/*` hash dirs is live for the current lock.)
