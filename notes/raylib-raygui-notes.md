# raylib / raygui notes for this repo

Working reference for the OCaml bindings (raylib/raygui 2.2.2, vendored
through `dune.lock`). Facts here were learned the hard way during GUI work;
each one either cost a debugging session or is not recoverable from the
binding's type signatures alone. Update it when you learn something new.

## Where the truth lives

- **The vendored C source is the only authoritative reference.** The binding
  is a thin ctypes wrapper over it; all geometry and styling questions are
  answered there:

  ```
  _build/_private/default/.pkg/raygui.2.2.2-*/source/src/c/vendor/raygui/src/raygui.h
  _build/_private/default/.pkg/raylib.2.2.2-*/source/src/c/vendor/raylib/src/
  ```

  `raygui.h` is a single-file implementation — controls, style system,
  `GuiDrawText`, `GetTextBounds` are all readable in it. When a widget
  renders strangely, read its C before theorizing.

- `raylib-ocaml-docs.txt` (repo root) is the generated binding API doc:
  signatures, module layout (`Raylib`, `Raygui`, submodules).

## Binding API shapes (the ones that matter)

```ocaml
Raygui.dropdown_box : Raylib.Rectangle.t -> string -> int -> bool -> int * bool
(*                bounds           options   active  edit_mode -> (index, toggle_pulse) *)

Raygui.toggle_group : Raylib.Rectangle.t -> string -> int -> int
(*                bounds            options   active -> active *)

Raygui.button : Raylib.Rectangle.t -> string -> bool

Raygui.get_style : Raygui.Control.t -> int
Raygui.set_style : Raygui.Control.t -> int -> unit
Raylib.get_color : int -> Color.t        (* 0x368bafff -> Color *)

Raylib.get_mouse_x / get_mouse_y : unit -> int
Raylib.is_mouse_button_pressed : MouseButton.t -> bool   (* MouseButton.Left *)
Raylib.check_collision_point_rec : Vector2.t -> Rectangle.t -> bool
```

- `Rectangle.create` takes **floats**; mouse getters return **ints**; cells
  drawn with raylib primitives (`draw_rectangle*`, `draw_text`,
  `measure_text`) take **ints** — convert at the boundary with
  `to_pixels`/`Float.of_int`.
- `Raygui.Control.t` is a unified variant: `` `Default of [ ... ] ``, `` `Toggle
  of [ prop | `Padding_toggle ] ``, `` `Button of prop ``, etc. `prop` covers
  the base properties (`` `Text_padding ``, `` `Text_alignment ``,
  `` `Border_width ``, colors). Not every control exposes every property —
  check the `Control` module in `raygui.mli` before assuming you can style
  something.

## The style system (the part that bites)

- **`GuiSetStyle(DEFAULT, prop, v)` propagates to every control** (base
  properties only). So `Fretboard_view.setup`'s 6px `` `Text_padding `` —
  installed for the dropdowns — also silently lands on toggles and buttons.
  Per-control `set_style` later overrides just that control.
- **`GuiGetStyle` has no fallback**: an unset control property is `0`
  (`LEFT` alignment for `` `Text_alignment ``, padding 0). The propagation
  above happens *at set time*, not at read time.
- **`GetTextBounds(control, bounds)` insets x by `BORDER_WIDTH +
  TEXT_PADDING` but does NOT shrink the width by the padding.** Net effect
  for centered text: the glyph is shifted right by `TEXT_PADDING` — with 6px
  padding, a snug box has its text pressed against the right border. This is
  why the chunk editor's `setup` zeroes `` `Toggle ``/`` `Button ``
  `` `Text_padding ``, and why its dropdown width budgets the padding.
- Defaults set at load (`GuiLoadStyleDefault`): `TEXT_SIZE = 10`,
  `TEXT_SPACING = 1`, `TEXT_ALIGNMENT = CENTER` (DEFAULT, propagated),
  `TEXT_PADDING = 0`, `BORDER_WIDTH = 1` (DEFAULT; **BUTTON = 2**, LABEL
  TEXT_ALIGNMENT = LEFT), `TOGGLE GROUP_PADDING = 2`,
  `DROPDOWNBOX ARROW_PADDING = 16`.

## Widget facts

- **`GuiToggleGroup` does NOT split its bounds.** Every entry is drawn at the
  full `bounds.width`, stepping right by `width + GROUP_PADDING`. So:
  bounds carry ONE entry's width; the control's footprint is
  `n * entry_width + (n-1) * GROUP_PADDING`. Layout math must use the
  footprint, the widget call the entry width. Max 32 entries.
- **`dropdown_box` returns a one-frame toggle pulse**, not the new value —
  XOR it against the threaded open-state or the box flashes open and shuts
  (`Fretboard_view.draw_controls`, `Chunk_view.draw_inspector`).
- The dropdown positions its text via `GetTextBounds(DEFAULT, bounds)` —
  only DEFAULT's `` `Text_padding `` reaches it; styling `` `DropdownBox ``
  padding is silently ignored (raygui 2.2.2).
- The dropdown arrow: glyph "v" in a 10px box at
  `bounds.x + bounds.width - ARROW_PADDING`.
- **A label starting with `#` is parsed as an icon spec (`#NNN#`)** by
  `GetTextIcon` before drawing. A bare `"#"` is fine (no trailing digits),
  but avoid `#`-prefixed control labels.
- `GuiDrawRectangle` draws the fill only when `color.a > 0` and the border
  only when `borderWidth > 0`.
- Widgets are immediate-mode and check `IsMouseButtonReleased` inside their
  bounds; a press elsewhere and release over a widget will still trigger it.

## Sizing formulas that work

All derived from the C above; kept in `bin/fretboard_view.ml` and
`bin/chunk_view.ml`.

- Dropdown width: `left_text_padding + widest_option + arrow_padding +
  text_to_arrow_gap` (the main bar uses 6 + text + 16 + 12). Budgets the
  right-shift from DEFAULT's `` `Text_padding ``.
- Toggle entry width: `widest entry text + padding` (12 works). Footprint:
  `n * entry_width + (n-1) * GROUP_PADDING`, with GROUP_PADDING read live
  via `` Raygui.Control.Toggle `Padding_toggle ``.
- Read style values live (`get_style`) instead of hard-coding defaults —
  styles are global and another view's `setup` may have changed them.
- For anything not a stock widget, plain raylib rectangles + text with
  `slot_at_point`-style float hit-testing keeps the geometry testable in
  `lib/layout`.

## OCaml 5.4 / Base gotchas hit along the way

- Comparison operators (`<`, `>`, `=`, ...) are **int-typed under
  `open! Base`** — scope float comparisons: `Float.(y < 0.)`.
- alcotest 1.9: `` Alcotest.(check (float 0.001)) `` — the `float` testable
  takes an epsilon; `Alcotest.(check float)` is a type error.
- `dune`'s `.pp.ml` files are **binary marshalled ASTs** (not readable text)
  in this toolchain — don't grep them; reproduce errors with the cached
  `ocamlopt.opt` and the include paths from a `dune build` error line
  (scratch experiments go in `scratch/`, which the root `dune` file excludes
  from the project).
