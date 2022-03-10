#!/bin/bash

URL="${1:-"http://localhost:8080/v1/"}"
BINDLE_SERVER="${2:-~/dev/rust/bindle/target/debug/bindle-server}"

export BINDLE_URL=$URL

echo "BINDLE_URL is set to '${BINDLE_URL}'"
echo "BINDLE_SERVER is set to '${BINDLE_SERVER}'"

# Do not forget to clean bindle's cache:
rm -rf ~/.cache/bindle

export RUST_LOG=debug,warp=info,bindle=trace
#export RUST_LOG=error,warp=info,bindle=debug

$BINDLE_SERVER --directory ${HOME}/.bindle/bindles --unauthenticated &


# client side
#export BINDLE_URL="http://localhost:8080/v1/"
#export RUST_LOG=error,warp=info,bindle=trace

#~/dev/rust/bindle/target/debug/bindle generate-label identity_input_output.json

#~/dev/rust/bindle/target/debug/bindle push-file -m application/json identity_model_bindle/0.1.0 identity_input_output.json

#~/dev/rust/bindle/target/debug/bindle info identity_model_bindle/0.1.0