#!/usr/bin/env bash

source $(dirname ${BASH_SOURCE[0]})/../../deploy/env

# Do not forget to clean bindle's cache:
rm -rf ~/.cache/bindle

$BINDLE_SERVER --directory ${HOME}/.bindle/bindles --unauthenticated


# client side
#export BINDLE_URL="http://localhost:8080/v1/"
#export RUST_LOG=debug,warp=info,bindle=trace

#~/dev/rust/bindle/target/debug/bindle generate-label identity_input_output.json

#~/dev/rust/bindle/target/debug/bindle push-file -m application/json identity_model_bindle/0.1.0 identity_input_output.json

#~/dev/rust/bindle/target/debug/bindle info identity_model_bindle/0.1.0