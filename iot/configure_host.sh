#!/usr/bin/env bash

# replaces --ctrl-host
export WASMCLOUD_CTL_HOST=192.168.178.134

export BINDLE_IP_ADDRESS_PORT=192.168.178.24:8080

wash ctl get inventory $(wash ctl get hosts -o json | jq -r ".hosts[0].id")

# curl --silent -T ../images/cat.jpg ${WASMCLOUD_CTL_HOST}:8078/mobilenetv27/matches | jq



