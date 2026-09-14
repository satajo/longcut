# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Longcut is a key-sequence based command executor for Linux on X11. Instead of multi-modifier shortcuts, users traverse a tree of named layers one keypress at a time, with a pop-up GUI panel showing the available options at each step.

Longcut is a resident process that owns its launch keys. Run without a subcommand, it binds the `keys_launch_global` and `keys_launch_app` configured under `x11: launcher:` on the X server through passive grabs and runs one navigation session per press. From the press on, the X server treats the keyboard as grabbed by longcut, so no key can be lost to another client; the session then takes the keyboard through the `Input` port and gives it back when the session ends. The keys held when the keyboard is taken are not modifiers of the keys that follow (`LaunchChord` in `longcut-x11`). A lock file in `$XDG_RUNTIME_DIR` keeps a second instance from starting. The X server releases every grab when the connection closes, so a crash cannot leave the keyboard grabbed.

## Build & Development Commands

Tooling comes from Nix (`nix develop` or the flake) and is orchestrated through Make.

| Command | What it does |
|---|---|
| `make check` | **Full verification** — nix flake check, nix format check, cargo check/fmt/clippy/test, plus integration tests in `longcut-application/` |
| `make build` | Release build (Nix + Cargo) |
| `make format` | Auto-format all code (Nix + Rust) |
| `cargo test -p <crate>` | Unit tests for one crate |
| `cargo test -p <crate> -- <test_name>` | A single test |
| `cargo clippy --all-targets -- -D warnings` | Lint; warnings are errors (see Lints under Conventions) |

Always finish with `make check` — passing unit tests alone is not sufficient.

The integration tests (`longcut-application/Makefile`) run the `check-config` subcommand against fixture configs in `longcut-application/test-data/` and grep the output for the expected error messages. `longcut-application/run-with-testing-config.sh` is a manual smoke-test launcher.

## Runtime Model

`longcut-application/src/main.rs` runs resident without a subcommand (bind the launch keys and run one navigation session per press, until killed) and defines one subcommand:

- `check-config` — validate the configuration file; exit code 1 on errors

## Architecture

Ports-and-adapters (hexagonal) architecture, enforced at the **crate** level. Crates are deliberately small and domain-driven: "If some concept feels wholly independent, and it could be made independent, then it should be made independent."

### Core rule

`longcut-core` depends on no other longcut crate. All external capabilities enter through five port traits in `longcut-core/src/port/`:

- `Launcher` — wait for the user's request for a session and report the mode asked for (the adapter owns the hotkeys and their configuration, nothing more)
- `Input` — take the keyboard and read keys from it; the returned `Keyboard` holds the keyboard exclusively until dropped, so core decides how long a grab lasts (today the whole session)
- `View` — render a data-only `ViewModel`
- `Executor` — run shell commands synchronously or in the background
- `WindowManager` — query the active window name (for app-specific layers)

### Adapter crates

An adapter crate bridges a port-declaring crate and an implementing crate. By convention it lives as a subdirectory of the **implementing** crate, named `adapter-<port-declaring-crate>`:

- `longcut-x11/adapter-longcut-core` — `X11Launcher` implements `core::port::Launcher` and owns the `x11:` configuration section, whose `launcher:` part holds the launch keys (deserialized through core's public `KeySchema`), `X11Input` implements `core::port::Input`, `X11WindowManager` implements `core::port::WindowManager`
- `longcut-gui/adapter-longcut-core` — `GuiView` implements `core::port::View`
- `longcut-shell/adapter-longcut-core` — `ShellExecutor` implements `core::port::Executor`
- `longcut-xcb/adapter-longcut-gui` — `XcbWindowManager` implements `gui::port::WindowManager` (a different trait from core's: it shows/hides the overlay window and hands the adapter's `CairoRenderer` to a render callback)

### Crate roles

- `longcut-core` — domain logic: layer navigation state machine, command execution, parameter input. The modes in `src/logic/` form a tree: each mode knows the modes it depends on, not its dependents. Entry point: `CoreService::run_forever(&dyn Launcher)`, which runs one session per launch. The `Input` port is a single blocking `capture_any()`.
- `longcut-gui` — presentation layer: converts `Screen`s into `Component` trees for `longcut-graphics-lib`.
- `longcut-graphics-lib` — backend-agnostic 2D rendering: `port::Renderer` trait plus components (`Column`, `Row`, `Table`, `Text`).
- `longcut-x11` — x11rb client: hotkeys bound through passive grabs (`Hotkeys`), the keyboard grab (`KeyboardGrab`) with the launch chord bookkeeping, key presses resolved through the X server's own keymap with xkbcommon, and active-window queries.
- `longcut-xcb` — overlay window management via x11rb + cairo-rs.
- `longcut-shell` — subprocess execution through `sh -c` with a configurable timeout.
- `longcut-config` — YAML config loading (serde_norway). Defines the `Module` trait.
- `longcut-application` — the main function. Wires all modules and adapters together; no domain logic.

### Wiring

All composition happens in `longcut-application/src/main.rs`. Modules reference each other through `&'a` borrows, not `Arc`, and everything runs synchronously on one thread. The instance lock is taken first, so a second instance exits before setting anything up. The GUI is built next, and every later startup failure (a configuration error, a launch key that cannot be bound) is shown on screen for a few seconds since a session-spawned process has no terminal. A keyboard grab that fails at a launch is shown the same way by `CoreService`, and the process keeps serving launches. Every configuration section is parsed before the launch keys are bound, so a configuration error never costs the user a grab. The launch keys are bound last and handed to `CoreService::run_forever`.

### ViewModel/Screen separation

`longcut-core` emits data-only view models; `longcut-gui` defines presentation-aware screens; `longcut-gui/adapter-longcut-core` converts between them.

### Key model

A `Key` (`longcut-core/src/model/key.rs`) is a `Symbol` plus modifiers. `Symbol` is a newtype named the way xkb names keysyms; it parses from and renders to that name. The xkb type behind it appears in core's API only in `Symbol::from_keysym` and `Symbol::keysym`, the doors for adapters that resolve or bind keys with xkb. Configuration spells a key as a single character or an exact, case-sensitive xkb keysym name (`Caps_Lock`, `Page_Up`), and modifiers as `Shift`, `Control`, `Alt`, `Super`. A press reports the keysym it produced and only the held modifiers xkb did not consume producing it: Shift+1 is `exclam` with no modifiers, Shift+F1 is `F1` with `Shift`, Control+Pause is `Break`. The Lock modifier is left out of the state a press resolves under, so Caps Lock cannot affect a shortcut. Matching is exact; there is no modifier-stripping fallback.

## Configuration

Default path is `<config_dir>/longcut/longcut.yaml` (i.e. `~/.config/longcut/longcut.yaml`), overridable with `--config-file`. Top-level YAML keys map to modules (`core:`, `gui:`, `shell:`, `x11:`); each module's config section is located and deserialized generically via `ConfigModule::config_for_module::<M>()` using `M::IDENTIFIER` as the key. The `Module` trait exists solely for this — it is not a runtime interface. A setting lives in the section of the module that interprets it: the launch keys are under `x11:` because only the X11 activation adapter reads them, while `keys_exit` and `keys_back` are under `core:` because the navigation loop does. A full example config lives in `examples/longcut.yaml`; per-module options are documented in `longcut-core/README.md`, `longcut-gui/README.md`, `longcut-shell/README.md`, and `longcut-x11/README.md`.

## Conventions

- Dependency versions and crate metadata live in the workspace root `Cargo.toml`; member crates must use `workspace = true` references instead of declaring their own.
- Rust edition 2024 across all crates.

## Lints

Lint levels are `warn` in the workspace `Cargo.toml` and the Makefile denies warnings, so every enabled lint is a build failure. `clippy::pedantic` is the only group; every other lint is opted in individually, with a comment stating the policy it enforces. A suppression is `#[expect(..., reason = "...")]` on the narrowest item, never `#[allow]`. Unsafe code is confined to the cairo/x11rb bridge in `longcut-xcb`, where each `unsafe` block carries its own `SAFETY` comment.
