# Ply 1.1.1 — spike notes

Gotchas hit while building `crates/spikes/ply` (stages 0–5, coming off the
xilem spike). Of the five candidates this is the closest to the old
raylib GUI: an immediate-mode macroquad frame loop, so the board is plain
macroquad drawing plus manual hit-testing, with Ply elements only for the
layout chrome.

## Boot

- `ply-engine = "1.1.1"` runs on a FORKED macroquad (`macroquad-ply` from
  git, `TheRedDeveloper/macroquad-fix`) re-exported through the prelude —
  do not add crates.io macroquad alongside it; `#[macroquad::main(conf)]`
  resolves through the re-export.
- Fonts: `FontAsset::Path(&'static str)` loads from disk at runtime — we
  point at `/System/Library/Fonts/Supplemental/Arial.ttf` rather than
  committing a font file (the `font!` macro embeds bytes relative to the
  crate manifest; needs a real file to embed). Board canvas text uses a
  second `load_ttf_font(FONT_PATH).await` macroquad `Font` — ply does not
  expose its loaded font.
- `Ply::new(&FONT).await`, then per frame: `clear_background`,
  `ply.begin()`, build elements, `ui.show(|_| {}).await` — note show
  REQUIRES the custom-element closure argument (the docs' `ui.show()` is
  main-branch syntax).

## The SKILL.md trap

The repo's `SKILL.md` documents MAIN, not the published 1.1.1:
`.with(|ui, el| ...)` (inline hover styling) and `Ply::set_cursor` do not
exist in 1.1.1. Verify any SKILL.md API against the crate source before
using it. `docs.rs` for the crate is thin; grepping the vendored source in
`~/.cargo/registry` is the reliable documentation.

## API notes

- Interaction is polling, not callbacks: give elements ids
  (`.id(...)`, `Id::new`, `Id::new_index("slot", i)`), then after
  `show()` check `ui.is_just_released(id)` / `ui.pointer_over(id)` /
  `ui.bounding_box(id)` (`BoundingBox { x, y, width, height: f32 }` in
  `ply_engine::math`). Callbacks like `.on_press` are `'static`, which
  would force `Rc<RefCell<..>>` state — polling is the immediate-mode way.
- Custom drawing: just draw with macroquad after `ui.show()`, positioned
  inside an element's `bounding_box` — no canvas widget needed because the
  frame is yours. `mouse_position()`/`is_mouse_button_pressed` for the
  board hit-testing; the element tree keeps layout responsibilities.
- `layout(|l| ...)`: `align(AlignX, AlignY)` — the X argument comes
  first, so vertically-centering a row is `align(Left, CenterY)`.
- `ui.text` takes `&str` (not `String`) in 1.1.1.
- The fork's `TextParams` has extra fields: `font_scale_aspect`,
  `rotation` — set both.
- Animations: the easing functions (`ease_out_cubic` etc.) and a `Lerp`
  trait ARE in the prelude, but there is no animation framework —
  everything is per-frame math against `get_time()`, exactly like the old
  raylib GUI. Full stage-5 set was straightforward as a result.
- Floating elements (the tonic dropdown) work: `.floating(|f| f
  .anchor((Left, Top), (Left, Top)).offset((140., 56.)).z_index(100))` —
  anchored to root coordinates, so offsets are hand-placed, not relative
  to the trigger button.
- Font/text color: element colors take `0xRRGGBB` ints; ply's `Color`
  float channels are 0..=255 space (not 0..1); macroquad drawing colors are
  the usual 0..1 `MacroquadColor`.

## Open runtime risks (unverified on screen)

- `high_dpi: true` + retina: ply layout coordinates vs macroquad drawing
  coordinates are assumed to be the same space; if hit-testing is offset
  on the user's screen, this is the first suspect.
- One-frame interaction lag: UI is built from state BEFORE polling this
  frame's clicks, so state changes land next frame (standard immediate
  mode, invisible in practice).
- Tab lines render in Arial (proportional) — expect misalignment; a
  monospace font asset would fix it.
