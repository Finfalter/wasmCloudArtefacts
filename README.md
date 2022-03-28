# MlInference

This repository provides a [wasmCloud](https://wasmcloud.dev/) capability provider and corresponding interface being designed to do __inference__ based on a given AI model.

## Prerequisites

### Bindle

For development and/or in case you want to avoid security issues, use [bindle v0.7.1](https://github.com/deislabs/bindle/tags). Use the latest version otherwise.

### Docker Compose

Make sure your Docker install has [Compose v2](https://docs.docker.com/compose/cli-command/#installing-compose-v2). See also [Install wasmCloud with Docker](https://wasmcloud.dev/overview/installation/install-with-docker/).

## Build

From the top-level **directory** build with `make`.

---
**NOTE**

As of __2022-03-24__, the application will not compile unless you additionally have a local copy of [wasmCloud/interfaces](https://github.com/wasmCloud/interfaces) checked out with branch __*feat/mlinference*__  __and__ the path of __*wasmcloud_interface_mlinference*__ matches its `/interfaces/ml/rust` in __Cargo.toml__. Expect that to be corrected soon!

---

## Deployment

General build artifacts are located in `/deploy`. Bindle specific build artifacts are located in `/bindle/models`. The script `/deploy/run.sh` drives the application. The application comprises a startup and shutdown of the following entities:

* nats
* registry
* bindle-server
* wasmCloud host

Type `./deploy/run.sh` for more information.

The script tries to launch __nats__ and __registry__ via `docker compose`. Following a recommendation from wasmCloud core-team the wasmCloud host is *not* started in a container. 

### Configuration

Start with a modification of paths in file `deploy/env`. This file is the only one which is necessary to modify in order to get a basic example up and running.

While starting up, the capability provider which comprises the inference engine tries to download artifacts from a bindle-server. You have to upload these artifacts first in order to make them downloadable by the capability provider. The following command uploads a pre-configured bindle. You have to call it only once.

```bash
./deploy/run.sh create-bindle
```

The definition of the bindle, `invoice_w_groups.toml`, and its two parcels, `identity_input_output.json` and `identity_input_output.onnx` are located in `/bindle/models`.

### Launch

Except the bindle-server, all entities are considered and started by using option `all`. Start the bindle-server first, then the other entities. To display all subcommands run `run.sh` without arguments.

```bash
cd deploy

# display all available subcommands
./run.sh

# start bindle-server
./run.sh bindle-start

# launch the application
./run.sh all

# execute next line to stop the application (except bindle server)
# ./run.sh wipe

# execute next line to stop bindle server
# ./run.sh bindle-stop
```

After a successful startup the *washboard* should look similar to the following screenshot:

<div style="width: 80%; height: 50%">

![washboard after successful launch](images/washboard.png "washboard after successful launch")

</div>

Once the application is up and running, start to issue requests. Currently, the repository comprises the following pre-configured models:

* __*identity*__ of ONNX format
* __*plus3*__ of Tensorflow format
* __*mobilenet*__ of ONNX format
* __*squeezenet*__ of ONNX format

## Examples

Apart from the underlying inference engine, e.g. ONNX vs. Tensorflow, the pre-configured models differ in a further aspect: concerning the *trivial* models, one may request processing upon arbitrary shapes of one-dimensional data, `[1, n]`. [Mobilenet](https://github.com/onnx/models/tree/main/vision/classification/mobilenet) and [Squeezenet](https://github.com/onnx/models/tree/main/vision/classification/squeezenet), however, have more requirements regarding their respective input tensor. To fulfill these, the respective input tensor of an arbitrary image may be preprocessed before being routed to the inference engine.

The application provides two endpoints. The first endpoint routes the input tensor to the related inference engine without any preprocessing. The second endpoint preprocesses the input tensor and routes it to the related inference engine thereafter:

1. `0.0.0.0:<port>/<model>`, e.g. `0.0.0.0:7078/identity`
2. `0.0.0.0:<port>/<model>/preprocess`, e.g. `0.0.0.0:7078/squeezenetv117/preprocess`

### Identity Model

To trigger a request against the __*identity*__ model, type the following:

```bash
curl -v POST 0.0.0.0:8078/model/identity/index/0 -d '{"dimensions":[1,4],"valueTypes":["ValueF32"],"flags":0,"data":[0,0,128,63,0,0,0,64,0,0,64,64,0,0,128,64]}'
```

The response should comprise `HTTP/1.1 200 OK` as well as `{"result":"Success","tensor":{"dimensions":[1,4],"valueTypes":["ValueF32"],"flags":0,"data":[0,0,128,63,0,0,0,64,0,0,64,64,0,0,128,64]}}`

The following happens:

1. The http __*POST*__ sends a request for a model with name *"challenger"*, index `0` and some `data`.
2. `data` is vector `[1.0f32, 2.0, 3.0, 4.0]` converted to a vector of bytes.
3. A response is computed. The result is sent back.
4. The `data` in the request equals `data` in the response because the pre-loaded model "*challenger*" is one that yields as output what it got as input.

### Plus3 model

To trigger a request against the __*plus3*__ model, type the following:

```bash
curl -v POST 0.0.0.0:8078/model/plus3/index/0 -d '{"dimensions":[1,4],"valueTypes":["ValueF32"],"flags":0,"data":[0,0,128,63,0,0,0,64,0,0,64,64,0,0,128,64]}'
```

The response is

```bash
{"result":"Success","tensor":{"dimensions":[1,4],"valueTypes":["ValueF32"],"flags":0,"data":[0,0,128,64,0,0,160,64,0,0,192,64,0,0,224,64]}}
```

Note that in contrast to the __*identity*__ model, the answer from __*plus3*__ is not at all identical to the request. Converting the vector of bytes `[0,0,128,64,0,0,160,64,0,0,192,64,0,0,224,64]` back to a vector of `f32` yields `[4.0, 5.0, 6.0, 7.0]`. This was expected: each element from the input is incremented by three `[1.0, 2.0, 3.0, 4.0]` &rarr; `[4.0, 5.0, 6.0, 7.0]`, hence the name of the model: __*plus3*__.

### Mobilenet model

```bash
# in order for the relative path to match call from directory 'deploy'
curl -v POST 0.0.0.0:8078/mobilenetv27/preprocess --data-binary @../providers/mlinference/tests/testdata/images/n04350905.jpg
```

Note that the output tensor is of shape `[1,1000]` and needs to be post-processed where the post-processing is currently not part of the application.

### Squeezenet model

```bash
# in order for the relative path to match call from directory 'deploy'
curl -v POST 0.0.0.0:8078/squeezenetv117/preprocess --data-binary @../providers/mlinference/tests/testdata/images/n04350905.jpg
```

Note that the output tensor is of shape `[1,1000]` and needs to be post-processed where the post-processing is currently not part of the application.

## Creation of new bindles

The capability provider assumes a bindle to comprise two parcels where each parcel is assigned one of the following two groups:

* __*model*__
* __*metadata*__

The first, `model`, is assumed to comprise model data, e.g. an ONNX model. The second, `metadata`, is currently assumed to be json containing the metadata of the model. In case you create new bindles, make sure to assign these two groups.

## Supported Inference Engines

The capability provider uses the amazing inference toolkit [tract](https://github.com/sonos/tract) and currently supports the following inference engines

1. [ONNX](https://onnx.ai/)
2. [Tensorflow](https://www.tensorflow.org/)

### Restrictions

Concerning ONNX, see [tract's documentation](https://github.com/sonos/tract) for a detailed discussion of ONNX format coverage.

Concerning Tensorflow, only TensorFlow 1.x is supported, not Tensorflow 2. However, models of format Tensorflow 2 may be converted to Tensorflow 1.x. For a more detailled discussion, see the following resources:

* `https://www.tensorflow.org/guide/migrate/tf1_vs_tf2`
* `https://stackoverflow.com/questions/59112527/primer-on-tensorflow-and-keras-the-past-tf1-the-present-tf2#:~:text=In%20terms%20of%20the%20behavior,full%20list%20of%20data%20types.`

Currently, there is no support of any accelerators like GPUs or TPUs. On the one hand, there is a range of [coral devices](https://coral.ai/products/) like the [Dev board](https://coral.ai/docs/dev-board/get-started) supporting Tensorflow for TPU based inference. However, they only support the [Tensorflow Lite](https://www.tensorflow.org/lite) derivative. For more information see Coral's [Edge TPU inferencing overview](https://coral.ai/docs/edgetpu/inference/).