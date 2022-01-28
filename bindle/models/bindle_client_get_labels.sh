#!/bin/bash

# server side

# Do not forget to clean bindle's cache:
# ~/.cache/bindle

#export BINDLE_URL="http://localhost:8080/v1/"
#export RUST_LOG=error,warp=info,bindle=trace
#export RUST_LOG=error,warp=info,bindle=debug
#~/dev/rust/bindle/target/debug/bindle-server --directory ${HOME}/.bindle/bindles --unauthenticated


# client side
export BINDLE_URL="http://localhost:8080/v1/"
export RUST_LOG=error,warp=info,bindle=trace

~/dev/rust/bindle/target/debug/bindle generate-label identity_input_output.json
~/dev/rust/bindle/target/debug/bindle generate-label identity_input_output.onnx

#~/dev/rust/bindle/target/debug/bindle push-file -m application/json identity_model_bindle/0.1.0 identity_input_output.json
#~/dev/rust/bindle/target/debug/bindle info identity_model_bindle/0.1.0