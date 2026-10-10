# Xilem (git main, Aug 2026) — spike notes

Gotchas hit while building `crates/spikes/xilem`. This is the only spike so
far that required reading the framework's own source (masonry's canvas and
button widgets) instead of docs — the published 0.4.0 (Oct 2025) predates
the `canvas` view entirely, so this spike pins `git = linebender/xilem`.

## Boot

- `Xilem::new_simple(state, app_logic, WindowOptions::new("title"))` then
  `.run_in(EventLoop::with_user_event())`.
- No animation framework at all — the whole stage-5 set is hand-rolled from
  a `task` view ticking every 16ms (`tokio::time::sleep` in a loop, sending
  `()` through a `MessageProxy`), plus `Instant` math in app state. The tick
  re-runs `app_logic` at 60fps forever, which is exactly the "no framework"
  cost the spike plan predicted.
- `task` closures must be non-capturing (`F: Fn(MessageProxy, &mut State) ->
  Fut`, asserted zero-sized); return a `Pin<Box<dyn Future + Send>>` from a
  named fn — a plain `impl Future` fn failed the HRTB bound with the
  classic "FnOnce is not general enough".
- `task` has `Element = NoElement`: it CANNOT sit inside a flex. Combine
  with `xilem::core::fork(element_view, elementless_view)`.

## Custom widget + view (the real work)

The checklist's board required the full custom-widget path:

- `impl Widget for FretboardWidget`: `type Action = BoardAction`;
  `measure` should mirror masonry's Canvas — `match len_req {
  LenReq::FitContent(space) => space, _ => Length::const_px(200.) }` to
  fill the flex; `layout(ctx, props, size)` stores the size and calls
  `ctx.set_clip_path` (paint is WIDGET-LOCAL, no origin math);
  `paint(ctx, props, painter)` draws; `on_pointer_event` hit-tests and
  calls `ctx.submit_action::<Action>(..)`.
- Pointer positions are `PhysicalPosition<f64>` — convert to kurbo `Point`
  by hand (untested on a HiDPI screen; may need scale-factor handling).
- Text: parley `lcx.ranged_builder(fcx, text, 1., true)` +
  `push_default(StyleProperty::FontSize(13.))` + `build(text)` +
  `break_all_lines(None)` + `align(None, Alignment::Start, ..)`, then
  `render_text(painter, Affine::translate((x, y)), &layout, &[color], true)`.
  Layouts are rebuilt per paint frame (~130 of them) — fine at this scale,
  memoize if it ever matters.
- Bridging the widget into xilem: `impl View<FretNot, (), ViewCtx>` with
  `type Element = Pod<W>`, built via `ctx.with_action_widget(|ctx|
  ctx.create_pod(..))`; `rebuild` syncs state into the widget through a
  `WidgetMut` static method; `message` handles actions by mutating
  `app_state` directly and returning `MessageResult::RequestRebuild`.
  Modeled on `xilem_masonry/src/view/canvas.rs`.
- `Painter::stroke(shape, &Stroke, brush)` — stroke style by reference,
  color as a THIRD argument (`Stroke::new(w).with_color(c)` does not exist
  in this kurbo version).
- Dynamic lists: `Vec` of views works as a flex child, but every element
  must come from the SAME closure literal (one literal = one type);
  the `+`/`-` buttons had to move out of the slots Vec into the tuple.

## Environment

- git-main xilem + fresh resolution currently breaks when merged with
  the workspace's resolution: blade-graphics pulls naga with its `wgsl`
  feature, and naga's error reporting doesn't compile against the
  codespan-reporting build the workspace produces (`String:
  WriteColor`). The standalone spike tree resolves around it. Nobody
  chased the exact feature-unification culprit; the practical answer was
  keeping the gpui spike (the other blade-graphics consumer) on its own
  lockfile, which incidentally kept xilem's tree healthy too.
- Docs are honest that it's alpha: the canvas view, the `imaging` Painter
  API, and the Widget trait shape all changed on main within the last
  months; anything written against it rots fast.
