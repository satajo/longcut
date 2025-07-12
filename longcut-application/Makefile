check: check-missing-config check-syntactically-invalid-config check-semantically-invalid-config check-valid-config

check-missing-config: $(LONGCUT_BIN)
	! $(LONGCUT_BIN) --check-config-only --config-file=test-data/no-such-file.yaml
	$(LONGCUT_BIN) --check-config-only --config-file=test-data/no-such-file.yaml 2>&1 | grep "Could not find configuration file"

check-syntactically-invalid-config: $(LONGCUT_BIN)
	! $(LONGCUT_BIN) --check-config-only --config-file=test-data/syntactically-invalid-config.yaml
	$(LONGCUT_BIN) --check-config-only --config-file=test-data/syntactically-invalid-config.yaml 2>&1 | grep "Failed to parse configuration file"

# TODO: Implement semantic config checks. This test should not pass.
check-semantically-invalid-config: $(LONGCUT_BIN)
	$(LONGCUT_BIN) --check-config-only --config-file=test-data/semantically-invalid-config.yaml

check-valid-config: $(LONGCUT_BIN)
	$(LONGCUT_BIN) --check-config-only --config-file=test-data/valid-config.yaml

LONGCUT_BIN = ../target/debug/longcut
$(LONGCUT_BIN):
	cargo build