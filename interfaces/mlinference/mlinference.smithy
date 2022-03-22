// mlinference.smithy

// Tell the code generator how to reference symbols defined in this namespace
metadata package = [ { namespace: "com.example.interfaces.mlinference", crate: "mlinference" } ]

namespace com.example.interfaces.mlinference

use org.wasmcloud.model#codegenRust
use org.wasmcloud.model#wasmbus
use org.wasmcloud.model#n
use org.wasmcloud.model#U8
use org.wasmcloud.model#U16
use org.wasmcloud.model#U32
use org.wasmcloud.model#U64
use org.wasmcloud.model#F16
use org.wasmcloud.model#F32
use org.wasmcloud.model#I32

//! The Mlinference service issues inference requests via an inference engine.
//! It exposes one method:
//!
//! - predict()

/// The Mlinference service
@wasmbus(
    contractId: "wasmcloud:example:mlinference",
    actorReceive: true,
    providerReceive: true,
    protocol: "2" )

service Mlinference {
  version: "0.1",
  operations: [ Predict ]
}

/// predict
operation Predict {
  input: InferenceRequest,
  output: InferenceOutput
}

@codegenRust(noDeriveDefault: true, noDeriveEq: true)
structure InferenceRequest {
  @required
  @n(0)
  model: String,

  @required
  @n(1)
  tensor: Tensor,

  @required
  @n(2)
  index: U32
}

/// The tensor's dimensions and type are provided as metadata to a model.
/// Any metadata shall be associated to the respective model in a blob store.
@codegenRust(noDeriveDefault: true, noDeriveEq: true)
structure Tensor {
    @required
    @n(0)
    tensorType: TensorType,

    @required
    @n(1)
    dimensions: TensorDimensions,

    @required
    @n(2)
    data: Blob
}

list TensorDimensions {
  member: U32
}

/// TensorType
@codegenRust(noDeriveEq:true)
union TensorType {
  @n(0)
  F16: U8,
   
   @n(1)
   F32: U8,

   @n(2)
   U8: U8,

   @n(3)
   I32: U8,
}

/// InferenceOutput
@codegenRust(noDeriveDefault: true, noDeriveEq: true)
structure InferenceOutput {
  @required
  @n(0)
  result: ResultStatus,

  @required
  @n(1)
  tensor: Tensor
}

@codegenRust(noDeriveEq:true)
structure ResultStatus {
  @required
  @n(0)
  hasError: Boolean,

  @n(1)
  error: MlError
}

union MlError {
  @n(0)
  invalidModel: U16,
    
  @n(1)
  invalidEncoding: U16,

  @n(2)
  corruptInputTensor: U16,

  @n(3)
  runtimeError: U16,

  @n(4)
  openVinoError: U16,

  @n(5)
  onnxError: U16,

  @n(6)
  tensorflowError: U16,

  @n(7)
  contextNotFoundError: U16
}