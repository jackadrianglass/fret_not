# Bogue notes for this repo

Working reference for bogue.20260208 (pinned in `dune.lock`). Facts here cost
either a build cycle or a vendored-source read; update it when you learn
something new. The bogue source is readable at
`_build/_private/default/.pkg/bogue.20260208-*/source/` — when the mli is
ambiguous, read the ml.

## Building and running

- tsdl needs system SDL2 via pkg-config: `devenv.nix` carries `pkgs.SDL2`,
  `pkgs.SDL2_ttf`, `pkgs.SDL2_image`, `pkgs.pkg-config` (lowercase `sdl2` is
  gone from the rolling nixpkgs pin — the attribute is `SDL2`). Inside
  `devenv shell` pkg-config resolves; a bare shell needs
  `PKG_CONFIG_PATH=.devenv/profile/lib/pkgconfig`.
- Launching always prints `[ERROR] Opam has not been initialised` — bogue
  probes for custom themes via opam, then falls back to the default theme.
  Harmless noise.
- `dune pkg lock` resolving bogue pulls `tsdl`, `tsdl-ttf`, `tsdl-image`,
  `ctypes`, `xdg`; the `conf-sdl2*` packages resolve as platform no-ops.

## Executables beside the raylib views

- With two `(executable)` stanzas in one directory, every stanza compiles ALL
  sibling modules by default — `bogue_spike` tried to link `tab_view.ml`
  (`open Fret_not`) and failed. Each stanza needs an explicit `(modules ...)`
  field (`(modules :standard \ bogue_spike)` for main).

## Windows and layout

- Bogue windows are created `windowed + resizable + hidden + opengl +
  allow_highdpi` (b_draw.ml `create_window`) — resizable by default.
- `Window.maximize_width` only works after `Main.run` physically creates the
  window; queue it with `Sync.push (fun () -> Window.maximize_width win)`
  before `Main.run`. There is NO public API for window minimum size — a gap
  vs raylib's `set_window_min_size`.
- `Layout.replace_room ~by:new_room old_room` replaces a room inside its house
  ("replace [room] by [by]"); the labeled arg is the replacement. This is how
  a `Select` with changed options gets rebuilt (its options are fixed at
  creation).
- All geometry inside an `Sdl_area` command should be computed from
  `Sdl_area.drawing_size` within the command itself, so a window resize
  re-renders correctly; an `L.on_resize canvas_room (fun () ->
  Sdl_area.update area)` is the belt-and-braces redraw trigger.

## Canvas text (the non-obvious part)

- Bogue draws widget text but has NO public text API for `Sdl_area`. Use
  `tsdl-ttf` directly inside the render queue (it runs after SDL init):
  `Ttf.render_utf8_blended font text color` → surface →
  `Sdl.create_texture_from_surface renderer surface` → `Sdl.render_copy
  renderer tex ~dst:(Sdl.Rect.create ~x ~y ~w ~h)` → destroy texture + free
  surface. Guard `Ttf.init` with `Ttf.was_init`.
- Font path: `Theme.get_font_path_opt "Ubuntu-R.ttf"` (bogue ships the font);
  scale its size with `Theme.scale_int` for Hi-DPI.
- tsdl 1.3.0 shape gotcha: `Sdl.query_texture` returns the size as a nested
  pair — `Ok (_, _, (w, h))`, not `Ok (_, _, w, h)`.
- Per-redraw texture churn is fine for tens of labels; for the real port,
  cache rendered textures or use `Sdl_area.cache`.

## API shapes that mattered

```ocaml
Sdl_area.add : t -> ?name:string -> (Tsdl.Sdl.renderer -> unit) -> unit
Sdl_area.add executes commands at render time, not at add time
Widget.sdl_area ~w ~h () : Widget.t   (* logical pixels *)
Widget.get_sdl_area : Widget.t -> Sdl_area.t
Select.create : ?dst:Layout.t -> ?action:(int -> unit) -> string array
  -> int -> Layout.t
Button (Widget.button) ~kind:Button.Switch ~action:(bool -> unit) : string -> t
Layout.flat ~sep ~margins / L.tower : Layout.t list -> Layout.t
L.resident : Widget.t -> Layout.t
Main.of_windows : Window.t list -> board;  Main.run board
Sync.push : (unit -> unit) -> unit   (* execute on the main thread *)
```
