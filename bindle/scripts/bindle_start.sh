#!/usr/bin/env bash

source $(dirname ${BASH_SOURCE[0]})/../../deploy/env

# Do not forget to clean bindle's cache:
rm -rf ~/.cache/bindle

$BINDLE_SERVER --directory ${HOME}/.bindle/bindles --unauthenticated &
