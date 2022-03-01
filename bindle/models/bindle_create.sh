#!/bin/bash

URL="${1:-http://localhost:8079/v1/}"
BINDLE_CLIENT="${2:-~/dev/rust/bindle/target/debug/bindle}"

# set some environment variables
export BINDLE_URL=$URL
export RUST_LOG=debug,warp=info,bindle=trace

#~/dev/rust/bindle/target/debug/bindle generate-label identity_input_output.json

# push an invoice
echo "pushing the invoice to bindle server .."
$BINDLE_CLIENT push-invoice invoice_w_groups.toml

# push invoice's parcels
echo "pushing parcels to bindle server .."
$BINDLE_CLIENT push-file -m application/json identity_model/0.2.0 identity_input_output.json
$BINDLE_CLIENT push-file -m application/octet-stream identity_model/0.2.0 identity_input_output.onnx

# retrieve information about the new bindle
echo "retrieving information from bindle server .."
$BINDLE_CLIENT info identity_model/0.2.0