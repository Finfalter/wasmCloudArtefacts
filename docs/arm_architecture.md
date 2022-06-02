# Build and deploy on ARM devices

This guide specifically addresses [__Coral dev board__](https://coral.ai/docs/dev-board/datasheet/) with its __Quad-core ARM Cortex-A53__. However, a deployment on other Arm based devices should be possible in a similar way.

Even though Coral dev board also disposes of an Edge TPU for accelerated inferencing, this guide currently addresses inferencing on the Arm cpu exclusively.

## Setup

> Note that all almost all steps to do in order to deploy on ARM are already implemented in branch `coral`.
>
> Details are explained here merely for completeness and transparency.

The overall setup still remains the same as shown in [Deployment and Provisioning](./index#deployment-and-provisioning). The peculiarity here is that whereas the __model repository__ on Bindle and the __OCI registry__ are still hosted on the same machine, the __runtime__ is deployed on the, potentially remote, Arm device.

## Installation

Given the machine learning application and corresponding tools are already installed on the development machine, for the setup to work it is necessary to further install [__wasmCloud__](https://wasmcloud.dev/) and [__NATS__](https://nats.io/) on the ARM device. The fastest way to install both is via a download from the respective release repository.

> To be executed on the ARM device

```bash
echo "Downloading NATS 2.8.1"
curl -fLO https://github.com/nats-io/nats-server/releases/download/v2.8.1/nats-server-v2.8.1-linux-arm64.tar.gz

echo "Downloading wasmCloud host 0.54.4"
curl -fLO https://github.com/wasmCloud/wasmcloud-otp/releases/download/v0.54.4/aarch64-linux.tar.gz

echo "Extracting..."
tar -xf aarch64-linux.tar.gz
tar -xf nats-server-v2.8.1-linux-arm64.tar.gz

# (optional)
sudo mv nats-server-v2.8.1-linux-arm64/nats-server /usr/local/bin/
```

## Compilation

The hardware target for Coral dev board is known as `aarch64`. All actors are inherently portable but the capability providers have to be compiled for their specific target.

The two capability providers in this application are __http-server__ and __mlinference__. __https-server__ is already available for `aarch64` but __mlinference__ has to be built. The recommended procedure is to cross compile the capability provider. The following steps guide through the sequence of cross-compilation.

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

The configuration is slightly more envolved. Related scripts allow to selectively deploy the machine learning application may either on the development machine or on the ARM device. 

### Network configuration

On the development machine in `deploy/env` there are the new environment variables `HOST_DEVICE_IP` and `TARGET_DEVICE_IP`. They represent the address of the development machine (host) and the ARM device (target device) respectively.

In case both parameters are not set, the application is going to be deployed on the development machine. In case `TARGET_DEVICE_IP` is set to the address of the ARM device, the application is going to be deployed remotely. In the latter case the value for `HOST_DEVICE_IP` should be set such that both addresses are in the same network.

Example values are

```bash
export HOST_DEVICE_IP=192.168.178.24
export TARGET_DEVICE_IP=192.168.178.148
```

### ARM device

Given `TARGET_DEVICE_IP` does not equal to `127.0.0.1` and `deploy/run_iot_device.sh` is launched, a checklist reminds of all points which should have been checked by now:

* set `HOST_DEVICE_IP` in `deploy/env`
* set `TARGET_DEVICE_IP` in `deploy/env`
* loaded `iot/configure_edge.sh` to the target device
* loaded `iot/restart_edge.sh` to the target device
* `source ./configure_edge.sh` on the target device
* started NATS server (`nats-server --jetstream`) on the target device
* started wasmCloud runtime (`restart_edge.sh`) on the target device

## Deployment

```bash
cd deploy
./run_iot_device.sh bindle-start
./run_iot_device.sh all
```

* * *
[back](./)