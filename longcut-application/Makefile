LONGCUT_BIN = ../target/debug/longcut

.PHONY: check
check: check-missing-config check-syntactically-invalid-config check-semantically-invalid-config check-valid-config

.PHONY: check-missing-config
check-missing-config:
	! cargo run -- --check-config-only --config-file=test-data/no-such-file.yaml
	cargo run -- --check-config-only --config-file=test-data/no-such-file.yaml 2>&1 | grep "Could not find configuration file"

.PHONY: check-syntactically-invalid-config
check-syntactically-invalid-config:
	! cargo run -- --check-config-only --config-file=test-data/syntactically-invalid-config.yaml
	cargo run -- --check-config-only --config-file=test-data/syntactically-invalid-config.yaml 2>&1 | grep "Failed to parse configuration file"

.PHONY: check-semantically-invalid-config
check-semantically-invalid-config:
	! cargo run -- --check-config-only --config-file=test-data/semantically-invalid-config.yaml
	cargo run -- --check-config-only --config-file=test-data/semantically-invalid-config.yaml 2>&1 | grep "Invalid configuration for module core"

.PHONY: check-valid-config
check-valid-config:
	cargo run -- --check-config-only --config-file=test-data/valid-config.yaml