// mlinference.smithy

// Tell the code generator how to reference symbols defined in this namespace
metadata package = [ { namespace: "com.pervaisive.interfaces.mlinference", crate: "mlinference" } ]

namespace com.pervaisive.interfaces.mlinference

use org.wasmcloud.model#wasmbus
use org.wasmcloud.model#U8
use org.wasmcloud.model#U32
use org.wasmcloud.model#U64

/// The Mlinference service 
@wasmbus(
    contractId: "example:interfaces:mlinference",
    actorReceive: true,
    providerReceive: true )

service Mlinference {
  version: "0.1",
  operations: [ Load, InitExecutionContext, SetInput ]
}

/// load
operation Load {
  input: LoadInput,
  output: LoadResult
}

/// init_execution_context
operation InitExecutionContext {
  input: Graph,
  output: IecResult
}

/// set_input
operation SetInput {
  input: SetInputStruct,
  output: BaseResult
}

/// compute
operation Compute {
  input: GraphExecutionContext,
  output: BaseResult
}

structure SetInputStruct {
    @required
    context: GraphExecutionContext,

    index: U32,

    @required
    tensor: Tensor
}

structure Tensor {
    @required
    dimensions: TensorDimensions,

    @required
    ttype: TensorType,

    @required
    data: TensorData
}

list TensorDimensions {
  member: U32
}

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

list TensorData {
  member: U8
}

structure LoadInput {
    @required
    builder: GraphBuilder,

    @required
    encoding: GraphEncoding,

    @required
    target: ExecutionTarget
}

/// see LoadInput
list GraphBuilder {
  member: U8
}

structure GraphEncoding {
  // enum seems to have no impact on the code generator
  @enum([
    {
        value: 0,
        name: "OPENVINO",
        documentation: """TBD""",
        tags: ["graphEncoding"]
    },
    {
        value: 1,
        name: "ONNX",
        documentation: """TBD""",
        tags: ["graphEncoding"]
    }
  ])
  @required
  encoding: U8
}

structure ExecutionTarget {
  // enum seems to have no impact on the code generator
  @enum([
    {
      value: 0,
      name: "EXECUTION_TARGET_CPU",
      documentation: """TBD""",
      tags: ["executionTarget"]
    },
    {
      value: 1,
      name: "EXECUTION_TARGET_GPU",
      documentation: """TBD""",
      tags: ["executionTarget"]
    },
    {
      value: 2,
      name: "EXECUTION_TARGET_TPU",
      documentation: """TBD""",
      tags: ["executionTarget"]
    }
  ])
  @required
  target: U8
}

structure Graph {
  @required
  graph: U32
}

@error("client")
structure GuestError {
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
  ])
  @required
  modelError: U8
}

@error("server")
structure RuntimeError {
  // enum seems to have no impact on the code generator
  @enum([
    {
      value: 0,
      name: "RUNTIME_ERROR",
      documentation: """TBD""",
      tags: ["MlInferenceError"]
    },
    {
      value: 1,
      name: "OPEN_VINO_ERROR",
      documentation: """TBD""",
      tags: ["MlInferenceError"]
    },
    {
      value: 2,
      name: "ONNX_ERROR",
      documentation: """TBD""",
      tags: ["MlInferenceError"]
    },
    {
      value: 3,
      name: "CONTEXT_NOT_FOUND",
      documentation: """TBD""",
      tags: ["MlInferenceError"]
    },
  ])
  @required
  runtimeError: U8
}

/// BaseResult
structure BaseResult {
  @required
  hasError: Boolean,

  runtimeError: RuntimeError,

  guestError: GuestError
}

/// LoadResult
structure LoadResult {
  @required
  result: BaseResult,

  @required
  graph: Graph,
}

/// InitExecutionContextResult
structure IecResult {
  @required
  result: BaseResult,

  @required
  gec: GraphExecutionContext,
}

structure GraphExecutionContext {
  @required
  gec: U32
}