# mlinference capability provider

This capability provider 
implements the "wasmcloud:example:factorial" capability


## Build the Capability provider

* from `providers/mlinference` execute `make`

## Run the tests

1. Start a __*NATS*__ server like `nats-server --jetstream`
	- if needed, install according to [these instructions](https://wasmcloud.dev/overview/installation/).
2. Run `cargo test` from `providers/mlinference`

