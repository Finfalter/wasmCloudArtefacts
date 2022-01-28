#!/bin/bash


# client side
export BINDLE_URL="http://localhost:8080/v1/"
export RUST_LOG=error,warp=info,bindle=trace

#~/dev/rust/bindle/target/debug/bindle generate-label identity_input_output.json

# push an invoice
~/dev/rust/bindle/target/debug/bindle push-invoice invoice_w_groups.toml

# push the first parcel
~/dev/rust/bindle/target/debug/bindle push-file -m application/json identity_model/0.2.0 identity_input_output.json

~/dev/rust/bindle/target/debug/bindle push-file -m application/octet-stream identity_model/0.2.0 identity_input_output.onnx

~/dev/rust/bindle/target/debug/bindle info identity_model/0.2.0