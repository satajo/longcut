#!/usr/bin/env bash
set -e -u -o pipefail

cd "$(dirname "${BASH_SOURCE[0]}")"

TIMEOUT="15s"

# A separate build step is needed before the timeout to prevent the build time from cannibalizing the run time.
cargo build

# Instead of using "cargo run", the built binary is executed directly. This avoids an implicit rebuild,
# where cargo would re-output build warnings etc, cluttering up the output.
echo "Running with a timeout of ${TIMEOUT}"
exec timeout $TIMEOUT ../target/debug/longcut --config-file test-data/testing-config.yaml
