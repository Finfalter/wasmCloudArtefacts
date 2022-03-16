#!/usr/bin/env bash

source $(dirname ${BASH_SOURCE[0]})/../../deploy/env

# client side

#~/dev/rust/bindle/target/debug/bindle generate-label identity_input_output.json

# push an invoice
$BINDLE push-invoice invoice_w_groups.toml

# push the first parcel
$BINDLE push-file -m application/json identity_model/0.2.0 identity_input_output.json

$BINDLE push-file -m application/octet-stream identity_model/0.2.0 identity_input_output.onnx

$BINDLE info identity_model/0.2.0