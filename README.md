# MlInference

This repository provides a [wasmCloud](https://wasmcloud.dev/) capability provider and corresponding interface being designed to do __inference__ based on a given AI model.

## Preliminary design decisions

1. Inference shall be __IN__ scope
	
> **_[Information]_**  Other actions which are typical for the discipline of machine learning shall deliberately be excluded, e.g. training and data exploration.

2. The capability provider shall be implemented based on ([Tract's](https://github.com/sonos/tract/tree/68db0209c9ffd1b91dff82884f4ae03b3622dd34)) [ONNX](https://onnx.ai/) inference engine in a first iteration. Models from nearly all famous ML frameworks can be ported to ONNX format, so the *coverage* should be acceptable from the start.

3. The interface shall mimic [WASI-NN](https://github.com/WebAssembly/wasi-nn).

4. Any AI model used in this context shall be packaged in a (separate) actor.

## Design of possible example applications

### Most basic design

![picture alt](http://via.placeholder.com/200x150 "Title is optional")


## Backlog / Roadmap

### Lifecycle of AI models and tensors

* Further support of multiple Inference Engines (IE) in named capability provider, e.g. OpenVino.

* Consumers of the capability provider (__cap__) can register artifacts. The artifacts are stored in the *capro*. However, currently, none of the stored artefacts is ever removed. There should be a mechanism to remove artifacts in the *capro*.