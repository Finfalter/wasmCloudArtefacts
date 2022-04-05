// imagenet.smithy
//

// Tell the code generator how to reference symbols defined in this namespace
metadata package = [ { 
  namespace: "org.wasmcloud.interface.mlimagenet", 
  crate: "wasmcloud_interface_mlimagenet" } ]

namespace org.wasmcloud.interface.mlimagenet

use org.wasmcloud.model#codegenRust
use org.wasmcloud.model#wasmbus
use org.wasmcloud.model#n
use org.wasmcloud.model#U8
use org.wasmcloud.model#U16
use org.wasmcloud.model#U32
use org.wasmcloud.model#F32
use org.wasmcloud.model#Unit
use org.wasmcloud.interface.mlinference#Tensor
use org.wasmcloud.interface.mlinference#Status
use org.wasmcloud.interface.mlinference#InferenceOutput

/// Description of Imagenet service
@wasmbus( 
  actorReceive: true,
  protocol: "2", 
)
service Imagenet {
  version: "0.1",
  operations: [ Postprocess ]
}

/// Converts the input string to a result
operation Postprocess {
  input: InferenceOutput,
  output: Matches
}

/// Classification
@codegenRust(noDeriveDefault: true, noDeriveEq: true)
structure Classification {
    @required
    @n(0)
    label: String,

    @required
    @n(1)
    probability: F32
}

list Matches {
    @n(0)
    member: Classification
}