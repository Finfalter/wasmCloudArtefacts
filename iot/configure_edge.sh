#!/usr/bin/env bash

# IOT device
# push with `mdt push <filename>`

export RUST_LOG=debug
export WASMCLOUD_OCI_ALLOWED_INSECURE=192.168.178.24:5000
export WASMCLOUD_RPC_TIMEOUT_MS=8000
export BINDLE_URL=http://192.168.178.24:8080/v1/
cd ~/wasmcloudHost