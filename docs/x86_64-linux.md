# Build and run on x86_64 Linux

## Structure

- [Build and run on x86\_64 Linux](#build-and-run-on-x86_64-linux)
  - [Structure](#structure)
  - [Prerequisites](#prerequisites)
  - [Build](#build)
  - [Configuration](#configuration)
    - [Prepare models](#prepare-models)
    - [Capability provider](#capability-provider)
  - [Deployment](#deployment)

## Prerequisites

The [**NATS**](https://nats.io/) and the local registry are served via a Docker container, see [installation notes](https://wasmcloud.dev/overview/installation/install-with-docker/). Also, make sure your Docker install has [Compose v2](https://docs.docker.com/compose/cli-command/#installing-compose-v2).

## Build

From the top-level directory build with `make`. This should complete without errors.

## Configuration

Update paths in file **deploy/env** to match your development environment.

Be sure to set `BINDLE` and `BINDLE_SERVER` in **deploy/env** to the paths to the bindle cli
and bindle server executables, respectively. If they are in your `$PATH`,
you can just set these to `bindle` and `bindle-server`. If you built
bindle from git, use the __0.7.1 tag__, run `cargo build`, and set
`BINDLE_HOME` to the path to the git repo.

### Prepare models

Models (in **bindle/models**) must be loaded into the bindle server.

If you are using your own model, you will need to create a "bindle invoice", a **.toml** file listing the bindle artifacts. Each artifact has a sha256 hash and file size of each artifact. See the existing toml files in **bindle/models** as examples.

### Capability provider

## Deployment

The script **deploy/run.sh** contains commands to run everything. In the
**deploy** folder, run **run.sh** to see a list of available subcommands.

Start the bindle server and load the models.

```bash
./run.sh bindle-start
./run.sh bindle-create
```

Start the local registry server, nats server, wasmcloud host,
actors, and providers. If this is your first time running running this
app, add `--console` to the end of the following command to open a new
terminal window with the host logs. The logs may be useful for
diagnosing any problems.

```bash
./run.sh all
# or, to open a $TERMINAL window with host logs
./run.sh all --console
```

* * *
[back](./)
