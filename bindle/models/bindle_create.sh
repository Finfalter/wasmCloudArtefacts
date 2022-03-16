#!/usr/bin/env bash

source $(dirname ${BASH_SOURCE[0]})/../../deploy/env

#~/dev/rust/bindle/target/debug/bindle generate-label identity_input_output.json

# push an invoice
echo "pushing the invoice to bindle server .."
$BINDLE push-invoice invoice_w_groups.toml

# push invoice's parcels
echo "pushing parcels to bindle server .."
$BINDLE push-file -m application/json identity_model/0.2.0 identity_input_output.json
$BINDLE push-file -m application/octet-stream identity_model/0.2.0 identity_input_output.onnx

# retrieve information about the new bindle
echo "retrieving information from bindle server .."
$BINDLE info identity_model/0.2.0