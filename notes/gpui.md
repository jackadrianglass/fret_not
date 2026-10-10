# GPUI 0.2.2 — spike notes

Status: RESOLVED, and the fix was not Xcode. The `runtime_shaders`
feature flag skips the build-time Metal shader compilation entirely:
build.rs only stitches the shader source, and the app compiles it
in-process at startup via `MTLDevice::new_library_with_source`
(gpui-0.2.2/src/platform/mac/metal_renderer.rs). So on macOS:

```toml
gpui = { version = "0.2.2", features = ["runtime_shaders"] }
```

builds with no Xcode, no `metal` toolchain — the devenv's apple-sdk is
enough. Costs: a few ms of shader compilation at app start instead of
build time. The naga/termcolor breakage seen earlier was a
workspace-resolution artifact; the spike's standalone dependency tree
resolves fine.

## What's implemented (now compiling)

- Board drawn via `gpui::canvas(prepaint, paint)` paint closures:
  `window.paint_quad` with the 0.2.2 `quad(bounds, corner_radii,
  background, border_widths, border_color, border_style)` constructor —
  6 args, unlike the 4-arg version newer docs show. Strings/nut/frets
  are thin quads, dots/halos are corner-radius circles, root halos are
  border-only quads.
- Hover/click: NO raw mouse math — transparent absolute overlay divs per
  dot (`.id().on_hover(...).on_click(...)`) using geometry cached from the
  previous frame's canvas paint (one frame of staleness on resize only).
- Labels/fret numbers as overlay text elements on the same cached
  geometry (canvas text in 0.2.2 is glyph-level only; skipped).
- Bloom: `with_animation` on the root element, id includes a generation
  counter so a key change re-mounts the animation. CRITICAL subtlety:
  with_animation repaints WITHOUT re-running render, so per-dot bloom
  factors must be computed inside the canvas paint from a shared
  `Rc<RefCell<f32>>` delta that the animator closure updates — values
  precomputed in render() would be frozen for the whole animation.
- Playhead/sweep/hover-ease: a `cx.spawn` loop ticking every 16ms via
  `cx.background_executor().timer`, updating `now` and `cx.notify()` only
  when something is active (playing/sweep/hover).
- Tonic dropdown: hand-rolled popup (absolute div list) — gpui core has no
  combobox/pick_list widget; that absence is itself a finding.

## API facts verified against docs.rs 0.2.2 (and compile fixes)

- Entry: `Application::new().run(|cx: &mut App| { cx.open_window(opts,
  |_window, cx| cx.new(|cx| State::new(cx))); cx.activate(true); })` —
  the root view is created with the `AppContext::new` trait method, NOT
  `cx.new_view` (that API arrived after 0.2.2).
- `canvas(prepaint: FnOnce(Bounds, &mut Window, &mut App) -> T, paint:
  FnOnce(Bounds, T, &mut Window, &mut App))`.
- `Animation::new(duration)` (+ `.repeat()`, `.with_easing(fn(f32) ->
  f32)`), `AnimationExt::with_animation(id, animation, |element, delta|
  element)` — implemented for anything IntoElement.
- Stateful divs: `.id(...)` unlocks `.on_click`, `.on_hover`,
  `.active(...)`, `.tooltip(...)` (StatefulInteractiveElement).
- `Window::paint_*`: paint_quad, paint_path, paint_glyph (baseline
  origin), paint_underline — no paint_text convenience.
- 0.2.2 gotchas hit while compiling: `hsla()` is not `const`; `Pixels/Pixels`
  does not exist (use `f32::from(pixels)`); `Pixels.0` is private;
  `.font()` takes a `Font` (`gpui::font("Menlo")`), not `&str`;
  `WeakEntity::update` in an async closure takes `cx` directly, not
  `&mut cx`; element ids from `&str` need `'static` lifetimes; and
  Rust-2024 capture rules make `cx.listener` closures borrow iterator
  items — deref Copy values out of iterators before capturing them.
- Runtime: a `gpui::canvas` measures to ~nothing unless you call
  `.size_full()` on it (it implements `Styled`) — the same
  "custom-drawing surface defaults to shrink" trap as iced's canvas
  missing `width(Fill)`. Symptom: the whole board collapses into a thin
  horizontal band.
- Coordinate spaces do not mix: canvas paint bounds and `window.paint_*`
  are WINDOW-space, while `.absolute().left()/.top()` on child divs are
  PARENT-relative. Positioning overlay elements from geometry computed
  in paint coordinates shifts them by the board's window origin — keep
  local-space accessors alongside the window-space ones.

## Scorecard implications

- Platform: the DEFAULT macOS build needs Xcode's Metal toolchain at
  build time, but `runtime_shaders` removes that requirement entirely —
  the devenv is enough. Linux uses Vulkan, Windows DirectX; no web.
- The crates.io release (0.2.2, Oct 2025) lags zed main significantly;
  ecosystem crates (gpui-motion etc.) say only the git version is
  supported. Building against git means a zed-monorepo checkout.
