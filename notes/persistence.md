# Persistence (words/document)

The `Document` (words + variant lineage + selected word + phrase steps) is
persisted as TOML; settings/session/sticky positions are deliberately not
persisted yet.

- Deps: `serde` (derive) is a direct dependency of `absolute` and `relative`
  (derives on `Alteration`, `ScaleDegree`, `DegreeReference`, `Slot`, `Word`);
  `gui` adds `serde`, `toml = "1"`, `dirs = "5"`. This intentionally relaxes the
  old "core is std-only" rule — chosen over gui-only DTOs so the core types
  serialize directly (orphan rule blocks deriving on them from gui).
- File: `<dirs::data_dir()>/fret_not/document.toml` — `$XDG_DATA_HOME` or
  `~/.local/share` (Linux), `~/Library/Application Support` (macOS),
  `%APPDATA%` (Windows). Resolved in `store::default_path()`.
- Shape (`store::StoredDocument`): `version = 1`, `selected_word`, then
  `[[words]]` tables (`word` slot array + optional `variant_of`) and
  `[[steps]]` tables. Field order matters — TOML wants scalars before arrays
  of tables. `Slot` uses default external tagging (`"rest"` /
  `{ note = { scale_degree = { degree, alteration }, octave } } }`);
  `Alteration` is snake_case (`flat`, `double_sharp`). Note: a word whose
  slots are all notes serializes as `[[words.word]]` array-of-tables, a mixed
  word as an inline array — both valid, both round-trip.
- Writes are atomic: serialize → write `document.tmp` in the same dir →
  `rename` over `document.toml`.
- Load: `App::new` returns `(App, Task)` with `Task::perform(store::load(),
  Message::DocumentLoaded)` (iced 0.14 tuple boot). Missing file → keep the
  seed document; parse failure or wrong `version` → the file is renamed to
  `document.bak` (quarantined, not deleted) and seeds are kept;
  `selected_word` is clamped to `words.len() - 1`, empty `words` is rejected.
- Save: `update` returns `Task<Message>`; every document-mutating arm ends
  in `state.save_document()` (one choke point — `refresh_document()` call
  sites), which fires `Task::perform(store::save(doc.clone()),
  Message::DocumentSaved)`. `Task` is `#[must_use]`, so tests bind
  `let _ = update(...)` and `let (mut app, _boot) = App::new()`.
- I/O is sync `std::fs` inside the async wrappers — the file is tiny and the
  future runs on the executor, never on the update thread (iced guidelines:
  no blocking work in `update`).
- Tests: inline in `crates/gui/src/store.rs`, using `temp_dir()` paths via the
  private `load_from`/`save_to` (the public `load`/`save` are thin wrappers
  over `default_path()`): round-trip, file shape, missing/corrupt/wrong-version
  quarantine, selection clamping.
