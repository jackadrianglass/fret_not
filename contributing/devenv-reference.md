# devenv reference

Quick notes on the dev environment this project uses. For anything not
covered here, go straight to the upstream docs linked in each section.

## What's configured

`devenv.nix`:

```nix
{ pkgs, lib, config, inputs, ... }:
{
  packages = [ pkgs.git pkgs.libffi pkgs.opam ];
  languages.ocaml = {
    enable = true;
    lsp.enable = true;
  };
}
```

- `languages.ocaml.enable` pulls in an OCaml toolchain and `dune`.
- `languages.ocaml.lsp.enable` (default `true`) sets up `ocaml-lsp` for
  editor integration.
- `pkgs.opam`, `pkgs.libffi`, `pkgs.git` are extra system packages —
  `libffi` is a build-time dependency of `ctypes`, which the `raylib`/
  `raygui` bindings use (see
  [`raylib-raygui-reference.md`](raylib-raygui-reference.md)).

**Important split:** devenv provides the *toolchain* (an `ocaml`/`dune`
binary, `opam`, LSP). It does **not** pin this project's actual dependency
versions (OCaml 5.4.1, `raylib`, `raygui`, `base`) — those come from dune's
own lockfile, `dune.lock/`. See
[`dune-reference.md`](dune-reference.md#what's-pinned-where). If a
dependency-version question comes up, look in `dune.lock/`, not
`devenv.nix`.

## Common commands

| Command | What it does |
|---|---|
| `devenv shell` | Enter the dev environment (puts `dune`, `ocaml`, `opam`, `ocaml-lsp` etc. on `PATH`) |
| `devenv up` | Start any declared background processes (none currently defined) |
| `devenv test` | Build the environment and run its checks — useful in CI |
| `devenv info` | Print environment info (useful for debugging a broken shell) |
| `devenv search <name>` | Search nixpkgs for a package to add to `packages` |

No `.envrc` is set up in this repo, so entering the environment is manual
(`devenv shell`) rather than automatic via `direnv` on `cd`.

## Where to look things up

| Question | Resource |
|---|---|
| Full options reference (`languages.*`, `packages`, `processes`, ...) | [devenv.sh/reference/options](https://devenv.sh/reference/options/) |
| OCaml-specific language support | [devenv.sh/supported-languages/ocaml](https://devenv.sh/supported-languages/ocaml/) |
| General docs / getting started | [devenv.sh](https://devenv.sh/) |
| Source / issues | [github.com/cachix/devenv](https://github.com/cachix/devenv) |
