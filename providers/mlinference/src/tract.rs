use std::cmp::Ordering;
use serde::{Deserialize, Serialize};
use wasmcloud_interface_mlinference::{Graph, GuestError};

// #[non_exhaustive]
// #[allow(dead_code)]
// pub struct GuestErrorWrap;

#[allow(dead_code)]
pub enum GuestErrorWrap {
    ModelError = 0,
    InvalidEncodingError = 1
}

impl From<GuestErrorWrap> for GuestError {
    fn from(gew: GuestErrorWrap) -> GuestError {
        match gew {
            GuestErrorWrap::ModelError => GuestError{model_error: 0},
            GuestErrorWrap::InvalidEncodingError => GuestError{model_error: 1},
        }
    }
}

#[non_exhaustive]
#[allow(dead_code)]
pub struct GraphEncoding;

#[allow(dead_code)]
impl GraphEncoding {
    pub const OPENVINO: u8 = 0;
    pub const ONNX:     u8 = 1;
}

#[non_exhaustive]
#[allow(dead_code)]
pub struct ExecutionTarget;

#[allow(dead_code)]
impl ExecutionTarget {
    pub const EXECUTION_TARGET_CPU: u8 = 0;
    pub const EXECUTION_TARGET_GPU: u8 = 1;
    pub const EXECUTION_TARGET_TPU: u8 = 2;
}

 #[derive(Copy, Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
 pub struct GraphWrap(u32);

impl From<GraphWrap> for u32 {
    fn from(e: GraphWrap) -> u32 {
        e.0
    }
}

impl From<GraphWrap> for Graph {
    fn from(w: GraphWrap) -> Graph {
        Graph{graph: w.0}
    }
}

impl From<u32> for GraphWrap {
    fn from(e: u32) -> GraphWrap {
        GraphWrap(e)
    }
}

impl From<Graph> for GraphWrap {
    fn from(g: Graph) -> GraphWrap {
        GraphWrap(g.graph)
    }
}

impl PartialOrd for GraphWrap {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GraphWrap {
    fn cmp(&self, other: &Self) -> Ordering {
        let (s, o) = (*self, *other);
        let s: u32 = s.into();
        let o: u32 = o.into();
        s.cmp(&o)
    }
}

// #[derive(Default)]
// pub struct State {
//     //pub executions: BTreeMap<GraphExecutionContext, TractSession>,
//     pub models: BTreeMap<Graph, Vec<u8>>,
// }

// impl State {
//     /// Helper function that returns the key that is supposed to be inserted next.
//     pub fn key<K: Into<u32> + From<u32> + Copy, V>(&self, keys: Keys<K, V>) -> K {
//         match keys.last() {
//             Some(&k) => {
//                 let last: u32 = k.into();
//                 K::from(last + 1)
//             }
//             None => K::from(0),
//         }
//     }
// }

// #[derive(Debug)]
// pub struct TractSession {
//     pub graph: TractGraph<InferenceFact, Box<dyn InferenceOp>>,
//     pub input_tensors: Option<Vec<TractTensor>>,
//     pub output_tensors: Option<Vec<Arc<TractTensor>>>,
// }

// impl TractSession {
//     pub fn with_graph(graph: TractGraph<InferenceFact, Box<dyn InferenceOp>>) -> Self {
//         Self {
//             graph,
//             input_tensors: None,
//             output_tensors: None,
//         }
//     }
// }