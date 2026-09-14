LONGCUT_BIN = ../target/debug/longcut

.PHONY: build
build:
	cargo build --quiet --package longcut-application

.PHONY: check
check: check-missing-config check-syntactically-invalid-config check-semantically-invalid-config check-unknown-key-config check-valid-config

.PHONY: check-missing-config
check-missing-config: build
	! $(LONGCUT_BIN) check-config --config-file=test-data/no-such-file.yaml >/dev/null 2>&1
	$(LONGCUT_BIN) check-config --config-file=test-data/no-such-file.yaml 2>&1 | grep -q "Could not find configuration file"

.PHONY: check-syntactically-invalid-config
check-syntactically-invalid-config: build
	! $(LONGCUT_BIN) check-config --config-file=test-data/syntactically-invalid-config.yaml >/dev/null 2>&1
	$(LONGCUT_BIN) check-config --config-file=test-data/syntactically-invalid-config.yaml 2>&1 | grep -q "Failed to parse configuration file"

.PHONY: check-semantically-invalid-config
check-semantically-invalid-config: build
	! $(LONGCUT_BIN) check-config --config-file=test-data/semantically-invalid-config.yaml >/dev/null 2>&1
	$(LONGCUT_BIN) check-config --config-file=test-data/semantically-invalid-config.yaml 2>&1 | grep -q "Invalid configuration for module core"

.PHONY: check-unknown-key-config
check-unknown-key-config: build
	! $(LONGCUT_BIN) check-config --config-file=test-data/unknown-key-config.yaml >/dev/null 2>&1
	$(LONGCUT_BIN) check-config --config-file=test-data/unknown-key-config.yaml 2>&1 | grep -q '"capslock" is not an xkb keysym name'

.PHONY: check-valid-config
check-valid-config: build
	$(LONGCUT_BIN) check-config --config-file=test-data/valid-config.yaml >/dev/null
