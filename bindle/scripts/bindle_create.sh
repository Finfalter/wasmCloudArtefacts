#!/usr/bin/env bash

source $(dirname ${BASH_SOURCE[0]})/../../deploy/env

# in case there are multiple models to push append lines like the next one
source ./push_bindle.sh ../models/identity_input_output.toml ../models/identity_input_output.csv
source ./push_bindle.sh ../models/plus3.toml ../models/plus3.csv
