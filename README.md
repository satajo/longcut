# Longcut

Key-sequence based executor

## Installation

Components of Longcut depend on the following system packages. Make sure that
they are installed before attempting the installation.

Fedora:

- gtk3-devel
- libX11-devel

Ubuntu:

- libgtk-3-dev
- libx11-dev

With the dependencies installed, build and install Longcut to the path using
Cargo with the following command.

    cargo install --path longcut-application/

After installation, Longcut is available under the command

    longcut

## Development

This project uses standard Rust ecosystem tooling. The project is built using
`cargo`, and code formatting is done using `rustfmt`.
