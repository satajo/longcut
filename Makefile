# All

.PHONY: build
build: nix-build rust-build

.PHONY: check
check: nix-check rust-check
	cd longcut-application && $(MAKE) check

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
	nix fmt . -- --ci

.PHONY: nix-format
nix-format:
	nix fmt .

# Rust

.PHONY: rust-build
rust-build:
	cargo build --release

.PHONY: rust-check
rust-check: rust-check-build rust-check-format rust-check-lint rust-check-doc rust-check-workspace-lints rust-check-unittest

.PHONY: rust-check-build
rust-check-build:
	cargo check

.PHONY: rust-check-format
rust-check-format:
	cargo fmt --check

.PHONY: rust-check-lint
rust-check-lint:
	cargo clippy --all-targets -- -D warnings

.PHONY: rust-check-doc
rust-check-doc:
	RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items

.PHONY: rust-check-workspace-lints
rust-check-workspace-lints:
	status=0; \
	for manifest in $$(find . -path ./target -prune -o -name Cargo.toml -print | grep -v '^\./Cargo\.toml$$'); do \
		if ! grep -A1 '^\[lints\]$$' "$$manifest" | grep -q '^workspace = true$$'; then \
			echo "$$manifest: missing [lints] workspace = true"; status=1; \
		fi; \
	done; \
	exit $$status

.PHONY: rust-check-unittest
rust-check-unittest:
	cargo test

.PHONY: rust-format
rust-format:
	cargo fmt