# Makefile for imagenetpreprocessor

PROJECT  = imagenetpreprocessor
VERSION  = $(shell cargo metadata --no-deps --format-version 1 | jq -r '.packages[] .version' | head -1)
REVISION = 0
# list of all contract claims for actor signing (space-separated)
CLAIMS   = wasmcloud:mlpreprocessing wasmcloud:builtin:logging

HOST_DEVICE_IP ?= 127.0.0.1

# registry url for our actor
REG_URL  = $(HOST_DEVICE_IP):5000/v2/$(PROJECT):$(VERSION)

# command to upload to registry (without last wasm parameter)
PUSH_REG_CMD = wash reg push --insecure $(REG_URL)

# friendly name for the actor
ACTOR_NAME = "imagenetpreprocessor"
# optional call alias for actor
ACTOR_ALIAS=mlinference/imagenetpreprocessor

include ./actor.mk

test::
	cargo clippy --all-features --all-targets
