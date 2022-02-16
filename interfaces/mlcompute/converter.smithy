// converter.smithy
//

// Tell the code generator how to reference symbols defined in this namespace
metadata package = [ { namespace: "org.example.interfaces.converter", crate: "mlcompute" } ]

namespace org.example.interfaces.converter

use org.wasmcloud.model#wasmbus
use org.wasmcloud.model#n
use org.wasmcloud.model#U8
use org.wasmcloud.model#U32
use org.wasmcloud.model#U64

/// Description of Converter service
@wasmbus( 
  actorReceive: true,
  protocol: "2" )
service Converter {
  version: "0.1",
  operations: [ Compute ]
}

/// Compute
operation Compute {
  input: ComputeRequest,
  output: ComputeOutput
}

structure ComputeRequest {
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
    @n(0)
    ttype: TensorType,

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
structure TensorType {
  // enum seems to have no impact on the code generator
  @enum([
    {
      value: 0,
      name: "TENSOR_TYPE_F16",
      documentation: """TBD""",
      tags: ["tensorType"]
    },
    {
      value: 1,
      name: "TENSOR_TYPE_F32",
      documentation: """TBD""",
      tags: ["tensorType"]
    },
    {
      value: 2,
      name: "TENSOR_TYPE_U8",
      documentation: """TBD""",
      tags: ["tensorType"]
    },
    {
      value: 3,
      name: "TENSOR_TYPE_I32",
      documentation: """TBD""",
      tags: ["tensorType"]
    }
  ])
  @required
  ttype: U8
}

/// ComputeOutput
structure ComputeOutput {
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
  err: U8
}