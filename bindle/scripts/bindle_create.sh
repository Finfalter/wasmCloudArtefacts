#!/usr/bin/env bash


_DIR=$(dirname ${BASH_SOURCE[0]})
source $(dirname ${BASH_SOURCE[0]})/../../deploy/env

# in case there are multiple models to push append lines like the next one
source ${_DIR}/push_bindle.sh ${_DIR}/../models/plus3.toml ${_DIR}/../models/plus3.csv
source ${_DIR}/push_bindle.sh ${_DIR}/../models/identity_input_output.toml ${_DIR}/../models/identity_input_output.csv
source ${_DIR}/push_bindle.sh ${_DIR}/../models/mobilenetv2-7.toml ${_DIR}/../models/mobilenetv2-7.csv
source ${_DIR}/push_bindle.sh ${_DIR}/../models/squeezenetv1-1-7.toml ${_DIR}/../models/squeezenetv1-1-7.csv
source ${_DIR}/push_bindle.sh ${_DIR}/../models/mobilenetv1_uint8_quant.toml ${_DIR}/../models/mobilenetv1_uint8_quant.csv
source ${_DIR}/push_bindle.sh ${_DIR}/../models/mobilenetv2_uint8_quant.toml ${_DIR}/../models/mobilenetv2_uint8_quant.csv
source ${_DIR}/push_bindle.sh ${_DIR}/../models/mobilenetv1_quant_edgetpu.toml ${_DIR}/../models/mobilenetv1_quant_edgetpu.csv
