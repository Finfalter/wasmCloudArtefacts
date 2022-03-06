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
//use org.wasmcloud.model#F16
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
    providerReceive: true )
//    protocol: "2" )

service Mlinference {
  version: "0.1",
  operations: [ Predict ]
}

/// predict
operation Predict {
  input: InferenceRequest,
  output: InferenceOutput
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
//@codegenRust(noDeriveDefault: true, noDeriveEq: true)
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

// /// TensorType
// union TensorType {
//   //  @n(0)
//   //  F16: F16,
   
//    @n(1)
//    f32: F32,

//    @n(2)
//    u8: U8,

//    @n(3)
//    i32: I32
// }

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

/// InferenceOutput
structure InferenceOutput {
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
  error: MlError
}

// union MlError {
//   @n(0)
//   INVALID_MODEL: U16,
    
//   @n(1)
//   INVALID_ENCODING: U16,

//   @n(2)
//   CORRUPT_INPUT_TENSOR: U16,

//   @n(3)
//   RUNTIME_ERROR: U16,

//   @n(4)
//   OPEN_VINO_ERROR: U16,

//   @n(5)
//   ONNX_ERROR: U16,

//   @n(6)
//   TENSORFLOW_ERROR: U16,

//   @n(7)
//   CONTEXT_NOT_FOUND_ERROR: U16
// }

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