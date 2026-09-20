# raylib / raygui (OCaml) reference

Quick notes on the bindings this project uses. For anything not covered here,
go straight to the upstream docs linked in each section.

## Packages

| opam package | provides | version locked |
|---|---|---|
| `raylib` | `Raylib` module — window, drawing, input, shapes, textures | 2.2.2 |
| `raygui` | `Raygui` module — immediate-mode GUI widgets, depends on `raylib` | 2.2.2 |

Both come from the OCaml bindings repo, not the C libraries directly — the C
sources are vendored, so no system SDL/raylib install is required.

- Bindings repo: https://github.com/tjammer/raylib-ocaml
- `raylib` on opam: https://opam.ocaml.org/packages/raylib/
- `raygui` on opam: https://opam.ocaml.org/packages/raygui/
- Upstream C libraries (for the underlying API these bindings mirror):
  - raylib cheat sheet: https://www.raylib.com/cheatsheet/cheatsheet.html
  - raygui repo/docs: https://github.com/raysan5/raygui

## Dune wiring

```dune
; dune-project (depends)
raylib
raygui
```

```dune
; bin/dune
(executable
 (public_name fret_not)
 (name main)
 (libraries fret_not raylib raygui))
```

Depending on `raygui` alone is enough to get both modules — it re-exports
`raylib`.

## Program shape

Every raylib-ocaml program follows the same setup → loop → close shape:

```ocaml
let setup () =
  Raylib.init_window width height "title";
  Raylib.set_target_fps 60

let rec loop state =
  match Raylib.window_should_close () with
  | true -> Raylib.close_window ()
  | false ->
      let open Raylib in
      begin_drawing ();
      clear_background Color.raywhite;
      (* draw + gui calls, compute next state *)
      end_drawing ();
      loop next_state

let () =
  setup ();
  loop initial_state
```

State is threaded through the recursive `loop`, not mutated in place —
raygui widgets return their new value each frame rather than taking a
callback (e.g. `Raygui.button rect label : bool` is `true` on the frame it's
clicked).

## Raygui widgets used / seen

All take a `Raylib.Rectangle.t` bounds as the first argument.

| function | signature (return) | notes |
|---|---|---|
| `label` | `Rectangle.t -> string -> unit` | static text |
| `button` | `Rectangle.t -> string -> bool` | `true` on the click frame |
| `check_box` | `... -> bool -> bool` | current value in, new value out |
| `window_box` | `Rectangle.t -> string -> bool` | `true` when its close button is clicked |
| `spinner` / `value_box` | `... -> int -> min:int -> max:int -> bool -> int * bool` | int + edit-mode flag |
| `text_box` / `text_box_multi` | `... -> string -> bool -> string * bool` | string + edit-mode flag |
| `combo_box` | `Rectangle.t -> string -> int -> int` | options as a `;`-separated string |
| `dropdown_box` | `... -> string -> int -> bool -> int * bool` | must be drawn last (can overlap other controls when open); see note below |
| `list_view` | `... -> string -> int -> int -> int * int` | active index, scroll index |
| `list_view_ex` | `... -> string list -> int -> int -> int -> int * int * int` | active, focus, scroll |
| `toggle_group` | `Rectangle.t -> string -> int -> int` | options `\n`-separated |
| `slider` / `slider_bar` | `... -> string -> float -> min:float -> max:float -> float` | |
| `progress_bar` | `... -> string -> string -> float -> min:float -> max:float -> float` | |
| `color_picker` | `Rectangle.t -> Color.t -> Color.t` | |
| `text_input_box` | modal dialog, returns `string * int` (text, button index) | |

Styling / control state:

- `Raygui.(set_style (Control field) value)` / `get_style` — e.g.
  `Raygui.(set_style (TextBox `Text_alignment) TextAlignment.(to_int Center))`
- `Raygui.set_state ControlState.(Normal|Focused|Pressed|Disabled)` — forces
  the visual state of controls drawn after it; reset to `Normal` when done.
- `Raygui.lock ()` / `Raygui.unlock ()` — disable input on controls drawn
  while locked (used for "grey out background while a modal is open").

`dropdown_box`'s returned `bool` is not the new edit-mode value — it's a
one-frame "toggle me" pulse (`true` on the click that opens it, and again
on the click/release that closes or selects an item), mirroring the C
signature `bool GuiDropdownBox(bounds, text, int *active, bool editMode)`.
The caller must XOR it against the edit-mode state it's threading, the
same as the upstream C usage `if (result) editMode = !editMode;`:

```ocaml
let active, toggled = Raygui.dropdown_box bounds text state.active state.open_ in
let open_ = if toggled then not state.open_ else state.open_ in
```

Assigning the returned bool directly as next frame's edit-mode (instead of
toggling) makes the box flash open for one frame and immediately snap
shut, since most frames return `false`.

`dropdown_box`'s text positioning also has a control-id quirk in raygui
2.2.2: its C implementation draws text via `GetTextBounds(DEFAULT, bounds)`,
not `GetTextBounds(DROPDOWNBOX, bounds)`. Setting `Text_padding` (or
`Border_width`) on `Control.DropdownBox` is silently ignored for text
positioning — only `Control.Default`'s value actually reaches it. Other
controls that draw their own text (e.g. `Label`) use their own control id
and aren't affected by changing `Default`.

Reference example this table was drawn from:
https://github.com/tjammer/raylib-ocaml/blob/main/examples/gui/controls_test_suite.ml

## Where to look next

- Full `Raygui` interface: `src/raygui/raygui.mli` in the bindings repo
- Full `Raylib` interface: everything under `src/` in the bindings repo
  (module list mirrors raylib.h — `Rectangle`, `Color`, `Vector2`, drawing
  functions, input functions, etc.)
- More examples (core window, shapes, textures, models): `examples/` in the
  bindings repo
