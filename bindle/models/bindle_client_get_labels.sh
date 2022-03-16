#!/usr/bin/env bash

source $(dirname ${BASH_SOURCE[0]})/../../deploy/env

# client side

$BINDLE generate-label identity_input_output.json
$BINDLE generate-label identity_input_output.onnx

#~/dev/rust/bindle/target/debug/bindle push-file -m application/json identity_model_bindle/0.1.0 identity_input_output.json
#~/dev/rust/bindle/target/debug/bindle info identity_model_bindle/0.1.0