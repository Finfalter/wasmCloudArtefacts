// mlpreprocessing.smithy
//

// Tell the code generator how to reference symbols defined in this namespace
metadata package = [ { 
  namespace: "org.wasmcloud.interface.mlpreprocessing",
  crate: "wasmcloud_interface_mlinference",
  } ]

namespace org.wasmcloud.interface.mlpreprocessing

use org.wasmcloud.model#codegenRust
use org.wasmcloud.model#wasmbus
use org.wasmcloud.model#n
use org.wasmcloud.model#U8
use org.wasmcloud.model#U16
use org.wasmcloud.model#U32
use org.wasmcloud.model#U64
use org.wasmcloud.model#F32
use org.wasmcloud.model#I32
use org.wasmcloud.model#Unit
use org.wasmcloud.interface.mlinference#Status
use org.wasmcloud.interface.mlinference#Tensor

/// Description of Mlpreprocessing service
@wasmbus( 
  actorReceive: true,
  protocol: "2", 
)
service MlPreprocessing {
  version: "0.1",
  operations: [ Convert ]
}

/// Converts the input string to a result
operation Convert {
  input: ConversionRequest,
  output: ConversionOutput
}

/// ConversionRequest
//@codegenRust(noDeriveDefault: true, noDeriveEq: true)
structure ConversionRequest {
  @required
  @n(0)
  data: Blob,

  // @n(1)
  // toType: TensorType,

  // @n(2)
  // toShape: TensorDimensions,
}

/// ConversionOutput
@codegenRust(noDeriveDefault:true)
structure ConversionOutput {
  
    @required
    @n(0)
    result: Status,

    @required
    @n(1)
    tensor: Tensor
}

/// Error returned with InferenceOutput
union MlPError {
    @n(0)
    runtimeError: String,

    @n(1)
    notSupported: String,
}