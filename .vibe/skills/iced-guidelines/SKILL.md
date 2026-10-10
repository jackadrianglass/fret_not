---
name: iced-guidelines
description: Load before any iced GUI work in this repo — adding features, screens, or widgets; changing update/view/subscription/Task code in crates/gui; GUI performance or animation work; investigating GUI crashes, panics, or misbehavior. Routes to the project's iced working rules.
---

# iced working rules (routing)

The source of truth is `notes/iced-guidelines.md` in the repo root. Read it
before starting the task; do not work from memory of past sessions.

- The doc is organized by task trigger — "Adding a feature or screen",
  "Writing update / view", "Async or slow work", "Performance work",
  "Something misbehaves or crashes". Read the section(s) matching the task;
  you don't need the whole file.
- API-level gotchas (signatures verified against the compiler) live in the
  companion `notes/iced.md` — consult it for concrete 0.14 API details.
- Before calling GUI work done, check the diff against the doc's
  Anti-patterns list.
- Rules marked [0.14] are bound to this iced version; after any iced
  upgrade, re-verify those rules before trusting them.
- The rules restate some house conventions from AGENTS.md on purpose; if
  they ever conflict, AGENTS.md wins.
