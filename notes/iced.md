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

- `pick_list` options and selection are any `T: Display + Clone` — typed
  payloads (e.g. `SpelledPitch`, a `Quality`-like enum) go straight into the
  list and come back typed in the message, no string round-tripping. Options
  can be a `Vec<T>` (or `const` array `.to_vec()`); selected is `Some(T)`.
- `use fretboard::fretboard;` shadows the crate name — `fretboard::tuning`
  then resolves against the module, not the crate. Import the functions
  (`use fretboard::fretboard::degree_at;`) instead.
- `Border` has a `radius` **field** (`border::Radius`), no builder method in
  this version.
- `button::Style { border: Border::default(), .. }` for outline-only
  styling; `Font::MONOSPACE` exists for tab alignment.
- Build pulls a `block v0.1.6` future-incompat warning through winit —
  transitive, not actionable from our side.

## Shared components (extracted during the layout rework)

- A view fn taking two or more `&` params and returning `Element<'_, M>`
  needs an explicit `'a` tied to all of them
  (`fn view<'a>(s: &'a S, d: &'a D) -> Element<'a, M>`); the anonymous
  `'_` cannot pick a param to borrow from.
- `Element::map` is the routing seam for a reusable component: a canvas
  `Program<ComponentEvent>` yields `Element<'_, ComponentEvent>`; each
  screen maps it into its own private `Message` (`.map(Message::Board)`),
  so the parent never sees the component's events directly.
- Have such a component take its overlay/config **by value**, not by
  reference — the built element then borrows only the long-lived state,
  and callers can pass a locally-constructed overlay without a
  "returns value referencing temporary" error.
- 0.14.2 widget forms verified in this codebase: `space::horizontal()`
  (pushes widgets apart in a row), `scrollable(content).horizontal()`
  (horizontal-only scroll, default scrollbar), container
  `.style(named_fn)` with a top-level `fn` for chrome backgrounds.
- Dead-code lint: an enum variant field that is only ever *constructed*
  (mapped into) but never *read* triggers `field is never read` — give the
  event a real consumer or destructure it in the match
  (`Message::Board(Event::Clicked(_))`).
- Dividers: `iced::widget::rule::horizontal(px)` / `rule::vertical(px)` —
  module functions in 0.14.2, **not** `horizontal_rule`/`vertical_rule`.
  `rule::Style` has four fields (`color`, `radius`, `fill_mode`, `snap`)
  and no `Default`; construct it fully. `.style(fn(&Theme) -> Style)`
  (no status param, unlike button/container).
- `iced::widget::Stack` centers a control group over a full-width bar:
  base layer = `row(...).width(Fill)`, overlay =
  `container(group).width(Fill).height(Fill).align_x/y(Center)`;
  a plain container passes clicks through to layers below, so the
  buttons under it stay clickable.

## Palette-driven theming (theme switcher)

- `Theme::ALL` is a `&'static [Theme]` of the 22 built-ins, ready for
  `pick_list(Theme::ALL.to_vec(), Some(state.theme.clone()), Message)`;
  `Theme` implements `Display` via `name()`.
- Runtime switching is free if every custom color derives from
  `theme.palette()` (background/text/primary/success/danger) inside
  widget style closures — iced passes `&Theme` to `button`/`container`/
  `rule`/`text` style functions at render time, so no view signature or
  cached-state plumbing is needed; a theme change is just a state field.
- `canvas::Program::draw` receives `&Theme` too — custom canvas painting
  can derive its colors there (`CanvasColors::of(theme)` in this codebase).
- `text(...).style(|theme| text::Style { color: Some(...) })` —
  `iced::widget::text::Style` has a single `color: Option<Color>` field;
  the catalog lives in `iced_core::widget::text`, re-exported.
- `button::Style` has a `text_color` field — use it instead of
  `text(...).color()` when the color should follow the theme (button
  styles get the theme; `.color()` needs a concrete `Color` at build
  time, which views don't have).
