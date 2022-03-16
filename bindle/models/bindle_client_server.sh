#!/usr/bin/env bash

source $(dirname ${BASH_SOURCE[0]})/../../deploy/env

# Do not forget to clean bindle's cache:
# ~/.cache/bindle

$BINDLE_SERVER --directory ${HOME}/.bindle/bindles --unauthenticated


# client side

$BINDLE generate-label identity_input_output.json

$BINDLE push-file -m application/json identity_model_bindle/0.1.0 identity_input_output.json

$BINDLE info identity_model_bindle/0.1.0