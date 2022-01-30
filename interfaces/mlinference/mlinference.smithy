// mlinference.smithy

// Tell the code generator how to reference symbols defined in this namespace
metadata package = [ { namespace: "com.example.interfaces.mlinference", crate: "mlinference" } ]

namespace com.example.interfaces.mlinference

use org.wasmcloud.model#wasmbus
use org.wasmcloud.model#n
use org.wasmcloud.model#U8
use org.wasmcloud.model#U32

//! The Mlinference service issues inference requests via an inference engine.
//! It exposes one method:
//!
//! - predict()

/// The Mlinference service
@wasmbus(
    contractId: "example:interfaces:mlinference",
    actorReceive: true,
    providerReceive: true )

service Mlinference {
  version: "0.1",
  operations: [ Predict ]
}

/// predict
operation Predict {
  input: InferenceRequest,
  output: InferenceResult
}

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
structure Tensor {
    @required
    dimensions: TensorDimensions,

    @required
    data: TensorData
}

list TensorDimensions {
  member: U32
}

list TensorData {
  member: U8
}

/// InferenceResult
structure InferenceResult {
  @required
  @n(0)
  result: ResultStatus,

  @required
  @n(1)
  tensor: Tensor
}

structure ResultStatus {
  @required
  @n(0)
  hasError: Boolean,

  @n(1)
  Error: MlError
}

structure MlError {
  // enum seems to have no impact on the code generator
  @enum([
    {
      value: 0,
      name: "MODEL_ERROR",
      documentation: """TBD""",
      tags: ["MlInferenceError"]
    },
    {
      value: 1,
      name: "INVALID_ENCODING_ERROR",
      documentation: """TBD""",
      tags: ["MlInferenceError"]
    },
    {
      value: 2,
      name: "CORRUPT_INPUT_TENSOR",
      documentation: """TBD""",
      tags: ["MlInferenceError"]
    },
    {
      value: 3,
      name: "RUNTIME_ERROR",
      documentation: """TBD""",
      tags: ["MlInferenceError"]
    },
    {
      value: 4,
      name: "OPEN_VINO_ERROR",
      documentation: """TBD""",
      tags: ["MlInferenceError"]
    },
    {
      value: 5,
      name: "ONNX_ERROR",
      documentation: """TBD""",
      tags: ["MlInferenceError"]
    },
    {
      value: 6,
      name: "CONTEXT_NOT_FOUND",
      documentation: """TBD""",
      tags: ["MlInferenceError"]
    },
  ])
  @required
  modelError: U8
}