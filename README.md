# MlInference

This repository provides a [wasmCloud](https://wasmcloud.dev/) capability provider and corresponding interface being designed to do inference based on a given AI model.

## Build the Capability provider

* from `providers/mlinference` execute `make`

## Build the Interface

* from `interface/mlinference` execute `make`

## Run the tests

1. Start a __*NATS*__ server like `nats-server --jetstream`
	- if needed, install according to [these instructions](https://wasmcloud.dev/overview/installation/).
2. Run `cargo test` from `providers/mlinference`

## Assumptions

### v0.1.0

* Smithy's `enum` is not supported by wasmCloud's code generator. 

## Backlog

### Lifecycle of AI models and tensors

* Consumers of the capability provider (__capro__) can register artifacts. The artifacts are stored in the *capro*. However, currently, none of the stored artefacts is ever removed. There should be a mechanism to remove artifacts in the *capro*.