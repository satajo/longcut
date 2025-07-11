# All

.PHONY: build
build: nix-build rust-build

.PHONY: check
check: nix-check rust-check

.PHONY: format
format: nix-format rust-format

# Nix

.PHONY: nix-build
nix-build:
	nix build

.PHONY: nix-check
nix-check: nix-check-build nix-check-format

.PHONY: nix-check-build
nix-check-build:
	nix flake check

.PHONY: nix-check-format
nix-check-format:
	nix fmt flake.nix -- --check

.PHONY: nix-format
nix-format:
	nix fmt flake.nix

# Rust

.PHONY: rust-build
rust-build:
	cargo build --release

.PHONY: rust-check
rust-check: rust-check-build rust-check-format rust-check-lint rust-check-unittest

.PHONY: rust-check-build
rust-check-build:
	cargo check

.PHONY: rust-check-format
rust-check-format:
	cargo fmt --check

.PHONY: rust-check-lint
rust-check-lint:
	cargo clippy -- -D warnings

.PHONY: rust-check-unittest
rust-check-unittest:
	cargo test

.PHONY: rust-check-unittest
rust-check-unittest-x11:
	cargo test --features x11-tests

.PHONY: rust-format
rust-format:
	cargo fmt