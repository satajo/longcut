# Longcut

Key-sequence based command executor for Linux on X11.

## Index

- [Introduction](#introduction)
- [Installation](#installation)
- [Configuration](#configuration)
- [Contributing](#contributing)
- [Developing](#developing)

## Introduction

Longcut is a key-sequence based command executor, which attempts to resolve the
usability conflict of having an ever-growing number of useful tools and commands
bound to the push of a button, but having an ever-decreasing number of free
buttons to bind to those tools and commands to. The way Longcut does this is by
allowing you to bind commands to  *semantically meaningful sequences of keypresses*.

As an example, in Windows the common *shortcut* for capturing a screenshot of a
selectable region is `Win + Shift + S`. To pull that off, you need to push down
*three* buttons at once and, worst of all, remember the whole key combination.

Now imagine instead that you could convert that intention into a sequence of steps,
after which the desired action would happen. This is exactly what Longcut allows you to
do.

The original intention:

```txt
I want to
take a Screenshot
of a Region
=>
(Screenshot is now available in the system clipboard)
```

Can, with the help of [scrot](https://github.com/resurrecting-open-source-projects/scrot),
be represented as a sequence:

```txt
Win (Start new sequence)
S (Screenshot)
R (Region)
=>
scrot --select - | xclip -selection c -t image/png
```

Which is in configuration looks like:

```yaml
core:
  keys_activate: Super_L
  layers:
    - name: Screenshot
      shortcut: s
      commands:
        - name: Region
          shortcut: r
          steps: scrot --select - | xclip -selection c -t image/png
```

Which is easily extended for capturing the whole screen or the active window:

```yaml
core:
  keys_activate: Super_L
  layers:
    - name: Screenshot
      shortcut: s
      commands:
        - name: Region
          shortcut: r
          steps: scrot --select - | xclip -selection c -t image/png
        - name: Screen
          shortcut: s
          steps: scrot - | xclip -selection c -t image/png
        - name: Window
          shortcut: w
          steps: scrot --focused - | xclip -selection c -t image/png
```

And so, instead of having to remember ever stranger combinations of modifier
keys, you can *one key at a time* traverse a sequence that is easy to remember
because it represents what you want to do. And if you can't remember (I often
forget!), the pop-up UI panel is always there to show you:

![Screenshot of Longcut GUI](media/gui.png)
(Contents shown in the screenshot are based on the [example configuration](examples/longcut.yaml).)

Now you know why Longcut exists and most of how it used. If you'd like to give
it a try, proceed to the next section for installation instructions.

## Installation

As of now, the only supported method of installation is by compiling from source.

### Compiling from source

Components of Longcut depend on the following system packages. Make sure that
they are installed before proceeding.

Fedora:

- gtk3-devel
- libX11-devel

Ubuntu:

- libgtk-3-dev
- libx11-dev

Clone the repository and run the following command in the repository root to
build and install Longcut.

```sh
cargo install --path longcut-application/
```

After installation, the Longcut binary is available under your cargo-bin-path
under the name `longcut`. Try running `longcut --help` to see if the binary
installed correctly.

Now that you have installed Longcut, you should read the next section about how
to configure it to do what you want.

## Configuration

(It's probably a good idea to open the [example configuration](examples/longcut.yaml)
in another tab when reading this section.)

Longcut is configured through a `yaml` configuration file. The default path for
this file is in the user's home config directory, in `~/.config/longcut/longcut.yaml`.
The configuration is parsed on startup, and any error in configuration file results
in an immediate startup error.

The Longcut codebase is structured into modules, which are responsible of reading
and parsing their own configurations. This is reflected in the configuration file
structure, as the top-level keys (`core:`, `gui:`, `shell:`, etc.) correspond to
these modules. The module-specific configuration sits under the module's key.

Each module's configuration is documented separately, and are listed below:

- [core](longcut-core/README.md) - Core logic: layer and command definitions, keybinds, etc.
- [gui](longcut-gui/README.md) - User interface: look and feel, fonts, colours, size, position, etc.
- [shell](longcut-shell/README.md) - Shell command execution: default_timeout.

And with that, that is all of the configuration. If you feel like there is
something missing, you may be interested in reading the next section about
how to contribute to Longcut.

## Contributing

First of all, a disclaimer. Longcut was born out of my personal wanting to have
a tool like it. As such, Longcut has a long history of being Done until it
suddenly isn't, and then being Done again. That has been the the cycle of
development all this time, and there is a good possibility that at the time
which you are reading this, Longcut is currently Done.

There two exceptions to the above. First of all, bugs and any obviously wrong
behaviour ought to be corrected. As Longcut is such an *important* interface to
me on the human-computer pathway, there is no reason to have it behave incorrectly.
Detailed and actionable bug reports are always welcome, as are their fixes.

Second, there exists the possibility that Longcut is not currently Done for *you*.
If that is the case, I do invite you to open an issue as a feature request or to
go develop and experiment in search of of improvements! If that search leads you
to a Done of your own, then I don't see a reason to not also make a pull request
about it.

That said, if you're going to contribute, you should also read the following
section to understand how and why that is done.

## Developing

This project uses standard Rust ecosystem tooling. The project is built using
`cargo`, tested using `cargo test`, and the code is formatted using `rustfmt`.
All dependency versions and crate metadata is centrally managed in the workspace
root, and crates should always refer to those instead of declaring their own
versions.

Longcut is structured around a [ports-and-adapters architecture](https://en.wikipedia.org/wiki/Hexagonal_architecture_(software)),
which is also reflected on the crate level. Each crate may export different port
definitions, represented by Rust traits, which are then implemented in special
adapter crates that bridge the domains of the port-declaring and the adapter-implementing
crates. By convention, these adapter crates reside under the directory of the
implementing side.

Extending the above, Longcut *heavily* utilises crates as boundaries to divide code
into minimally sized chunks based on somewhat fuzzy domain boundaries. The logic
behind the division can be summed up by "If some concept feels wholly independent,
and it could be made independent, then it should be made independent."

The entry point and the best place to start drilling downwards is the `longcut-application`
crate, which contains the main function. From there you can see how modules are
wired together, and begin to build an understanding.
