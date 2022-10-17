#!/usr/bin/env bash

# start bindle server!

# start NATs!


ps -ef | grep mlinference | grep -v grep | awk '{print $2}' | xargs kill
ps -ef | grep wasmcloud   | grep -v grep | awk '{print $2}' | xargs kill

export BINDLE_URL="http://localhost:8080/v1/"
export RUST_LOG=debug,warp=info,bindle=trace

cargo test