# Slint 1.18 — spike notes

Gotchas hit while building `crates/spikes/slint` (stages 0–5 of the GUI
spike plan, in one sitting, coming off the iced spike).

## UNRESOLVED: dot label legibility

The fretboard dot labels were reported too small to read, and raising them
did not visibly fix it:

- attempt 1: fixed `font-size: 10px` (unreadable)
- attempt 2: `min(14px, dot-r * 1.2)` with dot radius clamp 5–11px
  (still too small)
- attempt 3: `min(17px, dot-r * 1.1)` with dot radius clamp 6–16px,
  fret numbers 12px — still reported unreadable

For comparison, the iced spike's 10px canvas text on the same board was
readable, so this is not purely about the pixel size we set. Suspicions,
not verified:

- Slint's default font renders noticeably smaller than iced's default
  (Fira Sans) at the same px size; we never set `font-family` on the dot
  labels.
- HiDPI handling — everything else looked fine, so unlikely, but the
  labels never seemed to grow across restarts, which makes us wonder
  whether the `font-size` binding against `root.dot-r` (a property chain
  through the for-loop child) actually took effect. Verify with a fixed
  `font-size: 20px` before blaming scaling.

Next spike should not inherit this without a fixed-font-family test.

## Boot

- `slint = "1"` resolved to 1.18.1; needs `slint-build` as a build-dependency
  plus a one-line `build.rs` (`slint_build::compile("ui/app.slint")`) and
  `slint::include_modules!()` in main. No other ceremony.
- Rust ↔ DSL boundary: exported structs become Rust structs with
  snake_case fields; models are `ModelRc<T>`; `ModelRc::from(Rc::new(
  VecModel::from(vec)))` — `From<Vec<T>>` does NOT exist despite docs
  implying it ("from a slice or array" means `Rc<dyn Model>` paths).
- Callbacks register on the strong handle (`ui.on_x(...)`) with a weak
  handle captured into the closure (`ui_weak.unwrap()` inside).
- Property setters (`ui.set_*(...)`) exist for `in`/`in-out` properties.

## The DSL

- Ids are declared `name := Element`, NOT `id: name;`.
- Scoping is strict: unqualified references don't resolve to the component
  root. Everything wants `root.` (component) or `self.` (same element).
  `parent.width` works for the direct parent.
- **`public function` calls are "impure" and rejected inside property
  bindings** — geometry helpers must be inlined expressions. This is the
  single biggest structural difference from writing helpers.
- `color * float` does not exist — no color arithmetic. Dimming is done
  with a black overlay Rectangle at variable `opacity`.
- Ternaries need parens around the else-branch when it's a compound
  expression.
- `for item[index] in model` gives the index; `if` creates elements
  conditionally; `for d in root.dots: cell := Rectangle` names a loop
  child for back-reference.
- `animate width { duration: 120ms; easing: ease-out; }` on a nested
  element is the hover-ease pattern: the animated binding must depend ONLY
  on `touch.has-hover`, so tick-driven factors (bloom, pulse) live on the
  outer element and hover on the inner one.
- `Math.sin(360deg * tick / 1200 - fret * 15deg)` works in bindings;
  `min`/`max`/`mod` are builtins. `animation-tick()` exists but we drove
  time from a Rust `slint::Timer` (16ms) into an int `tick` property so
  Rust and the DSL share one epoch (bloom-start/play-start bookkeeping).

## Animation model vs iced

- Everything visual is a binding; time-driven animation = bind to `tick`
  and compute the factor per element. ~300 dot elements each with a tick
  binding redraws fine.
- Hover/ease is where Slint is genuinely better: `TouchArea.has-hover` +
  `animate` is one line, interruption/reversal handled by the runtime.
- The "bloom" stagger (delay by fret) was computed per-dot in the binding
  from `tick - bloom-start - fret * 18ms`; Rust just sets `bloom-start` on
  key change. No per-element animation objects like iced's
  `Animation<bool>` vec — the declarative version is less state to manage.

## Licensing reminder

Slint is GPL / royalty-free / commercial — pick a license lane before
building anything real on it.

## Misc

- struct field syntax: comma-separated; struct literal in expressions:
  `{ string-index: -1, fret: -1 }`.
- `Rectangle.border-radius: self.width / 2` makes circles; `border-width`/
  `border-color` for hollow/ring styles.
- std-widgets (`Button`, `CheckBox`, `ComboBox`) are fine for controls;
  the fretboard is hand-built because there is no canvas widget — the
  declarative element tree IS the custom drawing story.
