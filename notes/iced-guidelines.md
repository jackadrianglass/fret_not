# iced working rules

Rules of thumb for iced application work, refined from the Elm literature
(the architecture iced inherits) and iced's own material — sources at the
bottom. Companion to `notes/iced.md` (API gotchas) and to the house rules in
AGENTS.md; where a rule restates a house rule it says so, they should not
diverge.

Use: scan the section whose trigger matches the task; grep the
Anti-patterns list against a diff before calling GUI work done.

Version note: distilled against iced 0.14. The iced API is reworked freely
between releases (iced book, FAQ) — items marked [0.14] depend on this
release; re-verify them on any upgrade. Unmarked items are architecture
discipline and hold across versions.

## Adding a feature or screen

- Model the data first: define the `State` type, write down not just the
  possible states but the ones that must be impossible, and shape types so
  the impossible ones don't compile (iced book, First Steps; Feldman,
  "Making Impossible States Impossible").
- One concept, one newtype. Two kinds of names are two types, never two
  `String`s — the cross-wiring bug category is then eliminated, not tested
  away (0.19.3 post). Restates the house newtype style (`PitchClass`,
  `SpelledPitch`, ...).
- Invariant-holding values are unconstructable except through a validating
  function: the module exposes `validate` plus a getter, nothing else. The
  wrapper is zero-cost (0.19.3 post, `Email` example).
- Never-empty collection: mandatory head field plus `Vec` tail (0.19.3 post,
  `MinimalCycle a (List a)`).
- Enforce invariants with module privacy: fragile internal representation
  stays private; the only way to read it is a total conversion (0.19.3 post,
  `Graph`/`Component`).
- New interaction = new `Message` variant on the owning screen; the
  exhaustive match routes it. Never wire a mutating callback around the
  message flow (iced book, First Steps; SE Radio 713).
- Scale by splitting into screens; a screen reports up by returning an
  `Action` enum, the parent owns transitions; compose with `.map` so the
  parent never sees the child's message type (Pocket Guide, Scaling
  Applications). Skeleton below.
- Refactoring is cheap under this shape: change a variant or field and let
  the compiler enumerate every call site (Elm 0.15, Dreamwriter case).

## Writing update / view

- `update` is the only mutation site. iced's borrow checker enforces it:
  the runtime cannot call update while holding the widget tree, so a fresh
  view is produced after every mutation (SE Radio 713).
- Treat `view` and `subscription` as declarative, stateless functions; the
  runtime calls them after every batch of messages — never rely on when
  (iced book, FAQ).
- Initial state lives in `Default` next to the type, not in the boot path
  (iced book, The Runtime).
- Derived data that changes on discrete events (key change, selection) is
  computed in `update` and stored; view only interpolates what the frame
  needs.
- Keep view glue thin: widgets are generic, the app's state and messages
  are not — domain knowledge stays in the layer that owns it (iced book,
  Architecture).

## Async or slow work

- `update` runs on the main thread — anything slower than a frame returns a
  `Task` instead of running inline, or the whole UI freezes (SE Radio 713).
- Effects are values handed to the runtime: no threads spawned from update,
  no statics, no globals — anything the runtime can't see is invisible to
  debugging and replay (Elm guide; "Time Travel made Easy").
- External event streams enter via `Subscription` (`Subscription::run`,
  `Task::run`, `time::repeat`) (iced book, FAQ).

## Performance work

- Measure first with comet [0.14] (presentation + custom performance
  metrics) — guessing at renderer cost is how complexity creeps in
  (iced book, Philosophy).
- Exactly two optimization families wherever diffing happens: SKIP work
  (don't rebuild/diff a subtree whose inputs are unchanged —
  `canvas::Cache` for custom drawing) and ALIGN work (stable identities for
  list children so insert/remove doesn't reshuffle) (Czaplicki, "Blazing
  Fast HTML, Round Two").
- Perf fixes must live in view code. If a fix requires restructuring update
  or the model, stop — that's a design smell (Round Two).
- Renderer cost is allocation and lookup, not arithmetic: reuse caches,
  iterate in order, hold references instead of re-looking up (Round Two).
- Event batching and frame skipping are the runtime's job — emit messages
  freely, don't debounce the architecture from the app side (Round Two).
- Since 0.14, rendering is reactive [0.14]: only things that request redraw
  get drawn. Use the `Animation` API [0.14] for state-flowing tweens instead
  of hand-rolled interpolation; keep an always-on `time::every` subscription
  only while something is actually animating; canvas can schedule redraws
  without messages via `Action::request_redraw_at` (see notes/iced.md).

## Something misbehaves or crashes

- Total functions at boundaries: anything `head`-shaped on possibly-empty
  data returns `Option`, never panics (Elm 0.15). Restates the house rule —
  `panic!` only for invariant violations, documented on the function.
- Reproduce as a message list: state is a deterministic fold over messages,
  so any bad interaction is an exact message sequence. Prefer the v0.14
  toolset [0.14] to capture it — time-travel debugging, test recorder,
  runtime emulator, hot reload — over re-clicking by hand (0.14 release
  notes).
- Test update like the runtime does: feed a message sequence into a
  `State`, assert on the result — no window, no GUI init (iced book, The
  Runtime; inline-`#[cfg(test)]` house rule). Skeleton below.

## Anti-patterns (check against diffs)

- Blocking work in `update` (I/O, heavy compute) — return a `Task`.
- Recomputing event-driven derived data in `view` every frame.
- Mutating state outside `update` (thread, static, global) — breaks replay
  and time travel.
- Mutating callbacks instead of messages.
- A perf fix that touches domain logic.
- Relying on `view`/`subscription` call order or timing.
- Partial functions where `Option` fits.

## Skeletons

Sketches, not copy-paste — adapt names; not compiler-checked.

Screen composition (Pocket Guide pattern):

```rust
enum Screen {
    Board(Board),
    Settings(Settings),
}

enum Message {
    Board(board::Message),
    Settings(settings::Message),
}

// a screen keeps its Message private and reports upward
enum Action {
    None,
    Run(Task<board::Message>),
    OpenSettings,
}

impl Board {
    fn update(&mut self, message: board::Message) -> Action { /* ... */ }
    fn view(&self) -> Element<'_, board::Message> { /* ... */ }
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Board(message) => {
            let Screen::Board(board) = &mut state.screen else {
                return Task::none();
            };
            match board.update(message) {
                Action::None => Task::none(),
                Action::Run(task) => task.map(Message::Board),
                Action::OpenSettings => {
                    state.screen = Screen::Settings(Settings::default());
                    Task::none()
                }
            }
        }
        Message::Settings(message) => { /* same shape */ }
    }
}

fn view(state: &State) -> Element<'_, Message> {
    match &state.screen {
        Screen::Board(board) => board.view().map(Message::Board),
        Screen::Settings(settings) => settings.view().map(Message::Settings),
    }
}
```

Testing update (the runtime is a loop; the test is the loop):

```rust
#[test]
fn follows_a_key_change() {
    let mut state = State::default();

    update(&mut state, Message::Board(board::Message::TonicChanged(..)));

    assert_eq!(state.key.spelled(), expected_spelling);
}
```

## Sources

Digested: Czaplicki — "Blazing Fast HTML" (2014), "Round Two" (2016),
"Elm 0.15: Asynchrony with Tasks", "Another step towards Elm 1.0" (Oct
2026), guide.elm-lang.org (Architecture; Commands and Subscriptions);
James — "Time Travel made Easy" (2014); the iced book (Philosophy,
Architecture, First Steps, The Runtime, FAQ); Pocket Guide "Scaling
Applications"; iced 0.14.0 release notes; SE Radio 713 (transcript to
27:10; testing PRs #2698, #3059 in show notes).

Not yet digested (talks behind these rules; treat citations to them as
secondhand): Czaplicki — "Let's be mainstream!" (Curry On 2015),
"Controlling Time and Space" (2014), "The Life of a File" (Elm Europe
2017), "On Storytelling" (Deconstruct 2017, transcript online); Feldman —
"Scaling Elm Apps" (Elm Europe 2017), "Making Impossible States Impossible"
(elm-conf).
