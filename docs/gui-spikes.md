# GUI spike plan

Evaluate five Rust UI toolkits by rebuilding the fretboard view in each, then
pick one for the real GUI. One spike per toolkit, same feature checklist every
time, so the comparison is apples-to-apples.

The candidates:

1. [iced](https://docs.rs/iced/latest/iced/) — Elm-style, retained widgets,
   canvas widget for custom drawing
2. [Slint](https://docs.slint.dev/latest/docs/slint/) — declarative `.slint`
   DSL, compiled to native code, embedded heritage
3. [GPUI](https://gpui.rs/) — Zed's GPU-accelerated hybrid
   immediate/retained framework
4. [Xilem](https://docs.rs/xilem/latest/xilem/) — Linebender's reactive
   view-diffing layer over Masonry, alpha state
5. [Ply](https://plyx.iz.rs/) — young app engine (macroquad + Clay lineage),
   game-engine feature set, every platform from one codebase

The three questions each spike answers, in priority order:

1. How easy is it to get something up and running?
2. What can the animation story actually do?
3. What platforms does it reach?

## Ground rules

- One spike crate per toolkit, e.g. `crates/spikes/iced/`, added to the
  workspace, depending on the three core crates. Spike code is throwaway;
  don't polish it.
- All music knowledge comes from the core crates. A spike may only hold view
  state (selections, animation clocks, hover). If a spike needs to compute
  music itself, the core is missing a function — add it there instead.
- `cargo run` is human-run (per AGENTS.md): build and hand it over, the human
  judges the feel. Animation especially is a human-judged axis.
- Timebox each spike to a sitting. If the checklist isn't done when the box
  is up, stop, and note where.
- After each spike, write `notes/<toolkit>.md` (the library-notes convention)
  with the gotchas you wish you'd known at the start, and append a scorecard
  row (template below) to this document.

## The common feature checklist

Staged; each stage builds on the last. Record friction as you go, not at the
end — the friction is the data.

### Stage 0 — boot

- Window opens and shows something. Measure: minutes from empty crate to first
  pixel, cold and incremental build time, lines of code for hello-world.
- Note what the toolkit forced you to decide up front (event loop shape,
  state ownership, renderer backend, build tooling beyond cargo).

### Stage 1 — static fretboard

The core visual. Rebuild the previous GUI's fretboard view from
`docs/rust-migration-plan.md`:

- One horizontal line per string, light vertical line per fret, fret numbers
  below the lowest string. Fret count comes from `Instrument`, never a UI
  constant.
- Every playable position gets a dot, from `fretboard::degrees_in_window`:
  in-key = filled accent dot with centered degree label, degree-1 additionally
  ringed with a halo, off-key = hollow light-gray unlabeled circle.
- This stage is the custom-drawing test: the fretboard is ~90% bespoke
  drawing, so whatever canvas/path API exists gets a real workout. Note
  whether it felt like drawing (raylib-like) or like assembling primitives.

### Stage 2 — text and layout

- Control bar above the fretboard: tonic dropdown (12 spelled names), quality
  (Major/Minor), label-mode toggle (Degrees/Notes). These drive everything
  below through `Key`.
- Resizable window; geometry recomputed on resize, nothing anchored to
  hard-coded pixels.
- Text quality checks: dot labels centered inside circles (small text),
  the tonic dropdown (normal text), the title (styled text). Note font
  handling, unicode (flat/sharp signs), and text metrics pain.

### Stage 3 — interaction

- Changing tonic/quality re-derives the whole board — the core's
  tonic-anchored resolution means this is one `Key` value away.
- Hover a dot: it grows/glows. Note whether hover is per-widget or requires
  hit-testing your own canvas.
- Click a dot: selects that degree everywhere it appears (a cheap stand-in
  for the future pick-and-practice flow).

### Stage 4 — chunk and solver view

The data-heavy screen, aimed at the known not-yet-features:

- Slot strip: one cell per slot, degree label or `.` for rests, rests dimmed,
  selected cell with a cursor outline, click to select. `+`/`-` buttons
  append/remove slots.
- Pick-and-practice stub: run `chunk_solver::positions` on a hard-coded chunk
  (e.g. `[1, b3, 5, 6]` over a I–IV–V progression), prev/next buttons cycle
  the ranked fingerings, the current fingering renders as dots on the
  fretboard and as fret numbers in a simple tab (one line per string, open
  note labels at the left).
- This stage tests list/grid layout driven by dynamic data, and how the
  toolkit likes state that changes shape (slot counts), not just values.

### Stage 5 — animation

The fun axis. Four deliberately different kinds of animation, chosen because
together they cover the mechanisms a real practice app would need (feedback
on state change, time-driven playback, ambient motion):

- **Key-change bloom.** On tonic change, dots scale+fade in, staggered along
  the neck (leftmost first). Tests: enter/exit transitions on many elements
  at once, stagger, and whether an interruption mid-animation (user changes
  key again fast) reverses cleanly or fights the framework.
- **Chunk playhead.** A playhead sweeps the slot strip at a fixed tempo; each
  slot's notes pulse on the fretboard as it's crossed, rests skipped. Tests:
  time-driven animation from app logic (not just property tweens), running
  alongside interaction without jank.
- **Scale sweep.** All positions of one degree light up in a wave along the
  neck, looping, eased. Tests: custom/looping curves, paint-only animation
  (does the layout system stay out of the way?), continuous 60fps of many
  small moving elements.
- **Hover ease.** Stage 3's hover growth redone with a proper ease-out and
  interruption. Tests: the minimum-animation case, done the toolkit's way.

Score animation on: API shape (declarative vs hand-rolled clocks), easing
selection and custom curves, stagger support, interruption behavior, whether
animating a *custom-drawn* element is first-class or a second-class citizen.

## Scorecard

Append one row per spike. One sitting per row, not a research project.

| toolkit | first pixel | checklist stage reached | LOC (full checklist) | top 2 frictions | animation verdict | platform reach | verdict |
|---------|------------|------------------------|----------------------|-----------------|-------------------|---------------|---------|
| iced 0.14 | ~1 sitting incl. API research; compile-to-window minutes once API shape known | 5 (all) | ~945 (main 478, canvas 303, model glue 164) | canvas `Program` is 3 generics in 0.14 with `&Event`/`Action` indirection, docs lag the release; layout sizing — canvas needs explicit `width(Fill)` or it shrinks to intrinsic size | first-party `Animation<T>` tween API is clean (per-element stagger via `delay`, built-in easings + `Custom`), but canvas redraws still need a manual `time::every(16ms)` subscription driving `now` through state; enter-animated canvas content is DIY | desktop 3/3, wasm claimed (not tried) | strong baseline; Elm model maps well to the core's pure functions |
| Slint 1.18 | ~1 sitting, mostly DSL/compiler iteration; the DSL errors are precise and fast to fix | 5 (all) | ~996 (ui 480 DSL, main 470, model glue 164) | no canvas — the whole board is ~300 data-driven elements with inline geometry expressions (`public function` calls are rejected in bindings); no color arithmetic, dimming needs overlay elements | hover ease is one line (`TouchArea.has-hover` + `animate`) and interruption is runtime-handled — the best interaction animation of the three so far; bloom/playhead/sweep are tick-driven bindings, zero animation state in Rust; enter animations on model change are still DIY | desktop 3/3, embedded, wasm demo-grade (not tried) | declarative model is strong and resize is free; UNRESOLVED: dot labels unreadable even at 17px with enlarged dots (see notes/slint.md) — verify font-family/HiDPI before trusting this toolkit for the real GUI |
| GPUI 0.2.2 | compiles fine once you find `runtime_shaders` — the default build needs Xcode's Metal toolchain, but the feature flag compiles shaders in-process at startup; finding that took one sitting of build-script archaeology | 5 (all) | ~1088 (main 655, board 278, model glue 155) | docs lag the release badly (`new_view` doesn't exist in 0.2.2, `quad()` has a different signature than current docs); two runtime gotchas from the canvas model: it defaults to shrink (needs `.size_full()`), and paint is window-space while absolute children are parent-relative; its tree conflicts with the workspace's naga/codespan resolution so the spike builds standalone | `with_animation` wraps an element with a time-driven 0→1 delta and easing; retriggering = bumping the animation id. Canvas repaints without re-running render, so deltas must flow through shared state; tick-driven playhead/sweep via a spawned 16ms loop like the others | macOS/Linux/Windows, no web; iOS/Android via community wgpu layer | the smoothest-looking board of the five once running; the pre-1.0 API churn is the tax — keep, don't lead |
| Ply 1.1.1 | the fastest ramp of all five — the immediate-mode frame loop matches the old raylib GUI's shape, so the board was plain macroquad drawing plus manual hit-testing; SKILL.md + vendored source were enough | 5 (all) | ~904 (main 539, board 210, model glue 155) | `SKILL.md` documents main, not the release (`.with()` and `set_cursor` don't exist in 1.1.1); forked macroquad means no crates.io macroquad can coexist; the SKILL trap cost one compile cycle | no framework but the best ergonomics of the hand-rolled set: easing fns + `Lerp` in the prelude, per-frame math against `get_time()` felt like the old app; floating-element dropdown was the only real dropdown built in any spike | every platform incl. web/Android/iOS from one codebase (only the desktop trio built) | easiest for this exact app, but single maintainer + 6 releases + docs-ahead-of-release = highest long-term risk; 0BSD |
| Xilem (git main) | ~1 sitting, but the hardest of the five: the published 0.4.0 has no canvas view, so this pinned git main and went through full custom-widget plumbing (masonry Widget + View bridge), learning the API from the framework's own source since docs lag main | 5 (all) | ~977 (main 460, board widget+view 400, model glue 117) | alpha is real: Widget trait/`imaging` painter/parley text all changed on main within months; docs exist only for releases that don't have the canvas view; dynamic lists require same-closure-literal views; `task` views can't sit in a flex (need `fork`) | zero animation framework — all four stage-5 animations are `Instant` math driven by a 16ms `task` tick, which re-runs `app_logic` at 60fps forever. Felt like writing a small engine rather than using one | desktop 3/3 via Masonry, web via separate xilem_web API | architecturally interesting (view diffing + arbitrary state), but the real cost is that everything bespoke is DIY framework plumbing; a bet on the future, not a choice for today |
| (template) | minutes | 0–5 | | | 4 stage-5 features? how much fight? | what actually built, vs claimed | keep / drop |

## Results

All five spikes completed the checklist (GPUI after the `runtime_shaders`
unblock). The three questions the plan set out to answer:

**1. How easy is it to get something up and running?**

Ply was the fastest ramp — its immediate-mode macroquad loop is the same
shape as the removed raylib GUI, so the board was plain drawing plus
manual hit-testing. iced was a close second: the only spike whose
checklist worked nearly first-try, with one layout gotcha (canvas needs
`width(Fill)`). Slint's DSL errors were precise and fast to fix, but the
compiler is strict about scoping and rejects helper functions in
bindings. GPUI cost the most archaeology — its published docs describe a
future API (`new_view` doesn't exist in 0.2.2) and its build assumed
Xcode until the `runtime_shaders` feature was found. Xilem was the
hardest: no canvas view in the published release, so the board required
hand-writing a masonry Widget plus a custom View bridge against git
main, learning both from the framework's source.

**2. What can the animation story do?**

- Slint has the strongest framework: `animate` on any property, states
  with transitions, hover easing in one line with interruption handled
  by the runtime. Bloom/playhead/sweep still needed tick-driven bindings,
  but no animation state lived in Rust.
- iced's first-party `Animation<T>` tween API is clean (per-element
  stagger via `delay`, built-in easings plus `Custom`), but canvas
  redraws still require a manual `time::every` subscription.
- GPUI's `with_animation` is a time-driven 0→1 delta wrapper; because it
  repaints without re-running render, deltas must flow through shared
  state — usable, but the seams show.
- Ply and Xilem have no framework at all. Ply ships easing functions and
  `Lerp` in the prelude, which made its hand-rolled set the most
  pleasant; Xilem's was the plainest (a `task` view ticking 16ms and
  re-running `app_logic` at 60fps forever).

**3. Platform support?**

Desktop is table stakes for all five. Beyond that: Ply claims every
platform from one codebase (web/Android/iOS via the `plyx` CLI); Slint
adds embedded and demo-grade web plus mobile-in-progress; iced reaches
wasm; GPUI has no web target; Xilem's web story is a separate API
(`xilem_web`, DOM-based). Only the desktop trio was actually built —
web/mobile claims are unverified, per the plan.

### Recommendation

**iced.** For an app that is ~90% a custom-drawn fretboard plus rich
interaction over a pure functional core: the canvas widget maps to the
old raylib mental model, the Elm architecture maps to the core's pure
functions, the animation API is good enough, the checklist cost the
least total friction of any spike that didn't collapse into "write your
own framework plumbing," and it has wasm upside with MIT/Apache
licensing. Known costs: pre-1.0 with breaking releases, a book that is
still a WIP, and canvas repaints needing a tick subscription.

Ranking, with the reason each one lost:

1. **iced** — the pick.
2. **GPUI** — the best-rendering spike and smoothest board once running;
   keep as the fallback if performance ever becomes the constraint. Lost
   on published-docs drift and pre-1.0 churn.
3. **Slint** — the strongest animation and platform story; lost on the
   unresolved dot-label legibility (see notes/slint.md — verify
   `font-family` before ruling it out, a 30-minute test) and on the
   GPL/royalty-free/commercial license lane decision it forces.
4. **Ply** — the easiest spike for this exact app; lost on project
   risk: single maintainer, six releases, docs that describe unreleased
   APIs, a macroquad fork, and the youngest ecosystem.
5. **Xilem** — architecturally the most interesting; lost on present
   reality: alpha plumbing for anything bespoke, no animation framework,
   and a moving-target API. Revisit in a year.

## What we know going in

Researched up front so the spikes verify rather than discover. Verify each
claim during its spike; correct this section if something is stale.

### Platform matrix (claimed)

|          | Linux | macOS | Windows | Web/WASM      | Mobile            | Embedded |
|----------|-------|-------|---------|---------------|-------------------|----------|
| iced     | yes   | yes   | yes     | yes           | not a target      | no       |
| Slint    | yes   | yes   | yes     | demo-grade    | Android support in progress | yes (strength) |
| GPUI     | yes (Vulkan) | yes (Metal) | yes (DirectX) | no | community wgpu layer | no |
| Xilem    | yes   | yes   | yes     | via xilem_web (DOM, separate API) | not a target | no |
| Ply      | yes   | yes   | yes     | yes           | Android + iOS     | no       |

Only iced, Slint, and Ply claim web; only Slint and Ply claim mobile; only
Slint reaches embedded. Desktop is table stakes; web and mobile are pleasant
upside, not requirements. Treat web as a bonus stage: where the toolkit makes
it cheap, actually load the spike in a browser (iced's wasm target, Slint's
wasm build, `plyx`'s web target) and note the effort and the differences —
that demo-ability is part of the charm. Mobile stays claims-only; verify
nothing. Weight neither axis into the scorecard beyond a note.

### Animation claims

- **Slint**: the strongest on paper. First-class `animate` keyword on any
  property (duration, easing, delay, iteration), plus `states` with
  in/out transition blocks. Declarative, no clocks to manage. Verify it
  reaches inside custom-drawn elements, not just stock widgets.
- **GPUI**: `with_animation(id, Animation, closure)` with built-in easings
  (`ease_in_out`, `bounce`, `linear`), callback receives eased progress and
  re-renders until done. Time-driven 0→1; ecosystem crates (`gpui-motion`,
  `gpui-animation`) exist for springs/state transitions. Verify
  interruption and stagger ergonomics.
- **iced**: 0.14 shipped a first-party `Animation` API (plus reactive
  rendering and hot reloading); before that the community used `iced_anim`
  (spring/transition based). Newest of the mature three — verify how far it
  goes beyond stock widgets.
- **Xilem**: no animation framework yet — expect hand-rolled update
  driving (task/timer + view diff). Treat the stage-5 work as the honest
  measure of what living without framework animation costs.
- **Ply**: game-engine lineage (macroquad rendering, Clay-derived layout,
  custom shaders, sound, networking built in), so per-frame control should be
  trivial — the question is whether anything higher-level exists. Unknown
  going in; the spike finds out.

### Maturity and risk notes

- **iced**: mature, active, big ecosystem (awesome-iced), still
  pre-1.0/experimental label, breaking changes between releases. Elm
  architecture — messages, `update`, `view`.
- **Slint**: commercial backing, API-stability commitment, most production
  users. Licensing is the catch: GPL / royalty-free / commercial — pick a
  license lane before falling in love.
- **GPUI**: fastest renderer and powers a real editor, but pre-1.0, tied to
  Zed's repo/release schedule, docs thin (examples are the docs). A community
  fork (GPUI-CE) exists for framework-only releases. Heavy: expects async
  runtime patterns, entity/context model.
- **Xilem**: alpha, Linebender pedigree (Vello, Parley, AccessKit), the most
  architecturally interesting (view diffing like React with static-friendly
  types), but expect missing widgets and rough edges.
- **Ply**: released March 2026, single maintainer, ~6 versions, tiny
  download count. Highest variance: could be the most fun spike and the
  riskiest long-term bet. Immediate-mode-flavored builder API.

## Suggested order

1. **iced** — most mature, canvas widget maps to the old raylib mental
   model. Sets the baseline: whatever effort the checklist costs here is the
   reference point for every other spike.
2. **Slint** — the most different paradigm (DSL, designer-oriented) and the
   best animation claims; second so the checklist is still fresh.
3. **GPUI** — performance ceiling and worst docs; do it when you know the
   checklist well enough to see past toolkit friction.
4. **Ply** — wildcard; by now the checklist is well-worn, which is what a
   young toolkit needs to be judged fairly.
5. **Xilem** — alpha last; if its checklist run is respectable at this
   stage, that's a strong signal about the future.

## Out of scope for spikes

- Persistence of any state (still a deliberate core-level deferral).
- Audio, precise tempo timing, MIDI.
- Actually packaging for web/mobile. Web gets the bonus stage above (load
  the spike in a browser where cheap); beyond that, confirming the toolkit
  builds for a claimed target (`cargo check --target
  wasm32-unknown-unknown`) is enough. Mobile is claims-only. Shipping either
  is a later decision.
- Accessibility beyond noting what the toolkit gives for free — record it,
  don't build it.
