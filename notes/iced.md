# iced 0.14 — spike notes

Gotchas hit while building `crates/spikes/iced` (stages 0–5 of the GUI spike
plan, in one sitting). Verified against docs.rs and the compiler.

## Boot

- Entry: `iced::application(new, update, view)` builder. `new` is a
  `fn() -> State`. `.theme(...)` with a **named fn** — a closure `|_| Theme::X`
  fails the HRTB bound with a baffling "implementation of FnOnce is not
  general enough" pointing at the whole builder chain.
- `.default_text_size(...)` does not exist on the builder; set `.size()`
  per-text instead.
- `window::Settings { size, min_size, resizable, ..Default::default() }` —
  resizable defaults to false, set it explicitly.
- `iced::time::every` needs the `tokio` (or `smol`) cargo feature; `canvas`
  needs the `canvas` feature. Neither is on by default.

## Canvas

- `canvas::Program` in 0.14 is `Program<Message, Theme, Renderer>` — three
  generics, and you want the concrete `Theme`/`Renderer` aliases
  (`iced::Theme`, `iced::Renderer`), not a `Message`-only impl.
- `update(&self, state, event: &canvas::Event, bounds, cursor)` — event by
  **reference**, returns `Option<canvas::Action<Message>>`; publish messages
  via `Action::publish(msg)`. `Action` also has `request_redraw_at(Instant)`
  for animation without messages.
- `Cursor::position_in(bounds)` takes the rectangle **by value**.
- `canvas::Text` fields: `align_x` is `iced::widget::text::Alignment`,
  `align_y` is `iced::alignment::Vertical::Center`. `.size()` on `Text`
  widgets takes `Pixels` — `10.0.into()` infers either representation.
- No geometry cache used in the spike (every frame re-draws); `canvas::Cache`
  exists in `Program::State` if drawing ever becomes hot.

## Animation

- `iced::animation::Animation<T>` is a state-held tween: store it in app
  state, `.go(new_state, Instant)` (consuming) or `.go_mut` (in place) on
  change, `.interpolate(start, end, now)` (for `Animation<bool>`) or
  `.interpolate_with(closure, now)` in the view.
- `delay(d)` applies per-animation, so per-element stagger means one
  `Animation<bool>` per element (the spike keeps a `Vec` parallel to the
  dot list, rebuilt on key change — order stability between view and
  state is on you).
- Easings: full set incl. `EaseOutBack`, `EaseOutBounce`, and
  `Easing::Custom(fn(f32) -> f32)`.
- Canvas is custom drawing, so animation redraws are driven by an always-on
  `time::every(16ms)` subscription rather than anything reactive. The
  `Animation` API does the easing math; the tick subscription does the
  scheduling. Presumably stock widgets animate without a subscription.

## Misc

- `use fretboard::fretboard;` shadows the crate name — `fretboard::tuning`
  then resolves against the module, not the crate. Import the functions
  (`use fretboard::fretboard::degree_at;`) instead.
- `Border` has a `radius` **field** (`border::Radius`), no builder method in
  this version.
- `button::Style { border: Border::default(), .. }` for outline-only
  styling; `Font::MONOSPACE` exists for tab alignment.
- Build pulls a `block v0.1.6` future-incompat warning through winit —
  transitive, not actionable from our side.
