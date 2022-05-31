# Build and deploy on Arm devices

This guide specifically addresses [__Coral dev board__](https://coral.ai/docs/dev-board/datasheet/) with its __Quad-core Arm Cortex-A53__. However, a deployment on other Arm based devices should be possible in a similar way.

Even though Coral dev board also disposes of an Edge TPU for accelerated inferencing, this guide currently addresses inferencing on the Arm cpu exclusively.

## Setup

The overall setup still remains the same as shown in [Deployment and Provisioning](./index#deployment-and-provisioning). The peculiarity here is that whereas the __model repository__ on Bindle and the __OCI registry__ are still hosted on the same machine, the __runtime__ is deployed on the, potentially remote, Arm device.

## Compilation

The hardware target for Coral dev board is known as `aarch64`. All actors are inherently portable but the capability providers have to be compiled for their specific target. 

The two capability providers in this application are __http-server__ and __mlinference__. __https-server__ is already available for `aarch64` but __mlinference__ has to be compiled for this target.

## Configuration

## Deployment

* * *
[back](./)