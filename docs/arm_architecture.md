# Build and deploy on ARM devices

This guide specifically addresses [__Coral dev board__](https://coral.ai/docs/dev-board/datasheet/) with its __Quad-core ARM Cortex-A53__. However, a deployment on other Arm based devices should be possible in a similar way.

Even though Coral dev board also disposes of an Edge TPU for accelerated inferencing, this guide currently addresses inferencing on the Arm cpu exclusively.

## Setup

> Note that all almost all steps to do in order to deploy on ARM are already implemented in branch `coral`.
>
> Details are explained here merely for completeness and transparency.

The overall setup still remains the same as shown in [Deployment and Provisioning](./index#deployment-and-provisioning). The peculiarity here is that whereas the __model repository__ on Bindle and the __OCI registry__ are still hosted on the same machine, the __runtime__ is deployed on the, potentially remote, Arm device.

## Installation

Given the machine learning application and corresponding tools are already installed on the development machine, for the setup to work it is necessary to further install [__wasmCloud__](https://wasmcloud.dev/) and [__NATS__](https://nats.io/) on the ARM device.

>
>   TODO
>

## Compilation

The hardware target for Coral dev board is known as `aarch64`. All actors are inherently portable but the capability providers have to be compiled for their specific target.

The two capability providers in this application are __http-server__ and __mlinference__. __https-server__ is already available for `aarch64` but __mlinference__ has to be compiled for this target.

### Capability provider mlinference

* Make sure that `par_targets` in `providers/mlinference/provider.mk` comprises target `aarch64-unknown-linux-gnu`, e.g.

* in `providers/mlinference` create a file named `Cross.toml` with the following content:

```toml
[target.armv7-unknown-linux-gnueabihf]
image = "wasmcloud/cross:armv7-unknown-linux-gnueabihf"

[target.aarch64-unknown-linux-gnu]
image = "wasmcloud/cross:aarch64-unknown-linux-gnu"

[target.x86_64-apple-darwin]
image = "wasmcloud/cross:x86_64-apple-darwin"

[target.aarch64-apple-darwin]
image = "wasmcloud/cross:aarch64-apple-darwin"

[target.x86_64-unknown-linux-gnu]
image = "wasmcloud/cross:x86_64-unknown-linux-gnu"

[build.env]
passthrough = [
    "XDG_CACHE_HOME",
]
```

* Set the environment varialbe `XDG_CACHE_HOME` to the path the current user has write access, e.g. `XDG_CACHE_HOME=/tmp`

```bash
par_targets ?= \
   	aarch64-unknown-linux-gnu
```

* Eventually, in `providers/mlinference` build __mlinference__ with `make par-full`

## Configuration

## Deployment

* * *
[back](./)