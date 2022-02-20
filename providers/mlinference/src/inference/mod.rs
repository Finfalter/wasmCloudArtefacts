mod tract;
pub use tract::{TractSession, TractEngine, f32_vec_to_bytes, bytes_to_f32_vec};

use wasmcloud_interface_mlinference::{ Tensor, TensorType, InferenceOutput };

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{
    collections::{btree_map::Keys, BTreeMap},
    cmp::Ordering
};

/// Graph
#[derive(Copy, Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Graph(pub u32);

impl From<u32> for Graph {
    fn from(i: u32) -> Graph {
        Graph(i)
    }
}

impl From<Graph> for u32 {
    fn from(g: Graph) -> u32 {
        g.0
    }
}

impl PartialOrd for Graph {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Graph {
    fn cmp(&self, other: &Self) -> Ordering {
        let (s, o) = (*self, *other);
        let s: u32 = s.into();
        let o: u32 = o.into();
        s.cmp(&o)
    }
}

/// GraphEncoding
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct GraphEncoding(pub u8);

impl GraphEncoding {
    pub const GRAPH_ENCODING_OPENVINO: u8 = 0;
    pub const GRAPH_ENCODING_ONNX:     u8 = 1;
}

// /// GraphBuilder
// #[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
// pub struct GraphBuilder(pub Vec<u8>);

/// GraphExecutionContext
#[derive(Copy, Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct GraphExecutionContext(u32);

impl From<u32> for GraphExecutionContext {
    fn from(i: u32) -> GraphExecutionContext {
        GraphExecutionContext(i)
    }
}

impl From<GraphExecutionContext> for u32 {
    fn from(gec: GraphExecutionContext) -> u32 {
        gec.0
    }
}

impl PartialOrd for GraphExecutionContext {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GraphExecutionContext {
    fn cmp(&self, other: &Self) -> Ordering {
        let (s, o) = (*self, *other);
        let s: u32 = s.into();
        let o: u32 = o.into();
        s.cmp(&o)
    }
}

/// ExecutionTarget
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ExecutionTarget(pub u8);

impl ExecutionTarget {
    pub const EXECUTION_TARGET_CPU: u8 = 0;
    pub const EXECUTION_TARGET_GPU: u8 = 1;
    pub const EXECUTION_TARGET_TPU: u8 = 2;
}

/// TensorType
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct TType(pub u8);

impl TType {
    pub const F16: u8 = 0;
    pub const F32: u8 = 1;
    pub const  U8: u8 = 2;
    pub const I32: u8 = 3;
}

impl From<TensorType> for TType {
    fn from(tt: TensorType) -> TType {
        TType(tt.ttype)
    }
}

impl From<TType> for TensorType {
    fn from(tt: TType) -> TensorType {
        TensorType{ttype: tt.0}
    }
}

#[derive(Default)]
pub struct ModelState {
    executions: BTreeMap<GraphExecutionContext, TractSession>,
    models: BTreeMap<Graph, Vec<u8>>,
}

impl ModelState {
    /// Helper function that returns the key that is supposed to be inserted next.
    pub fn key<K: Into<u32> + From<u32> + Copy, V>(&self, keys: Keys<K, V>) -> K {
        match keys.last() {
            Some(&k) => {
                let last: u32 = k.into();
                K::from(last + 1)
            }
            None => K::from(0),
        }
    }
}

/// InferenceEngine
#[async_trait]
pub trait InferenceEngine {
    async fn load(&self, builder: &[u8], encoding: &GraphEncoding, target: &ExecutionTarget) -> InferenceResult<Graph>;
    async fn init_execution_context(&self, graph: Graph) -> InferenceResult<GraphExecutionContext>;
    async fn set_input(&self, context: GraphExecutionContext, index: u32, tensor: &Tensor) -> InferenceResult<()>;
    async fn compute(&self, context: GraphExecutionContext) -> InferenceResult<()>;
    async fn get_output(
        &self,
        context: GraphExecutionContext,
        index: u32
    ) -> InferenceResult<InferenceOutput>;
    async fn drop_model_state(&self, graph: &Graph, gec: &GraphExecutionContext);
}

/// InferenceResult
pub type InferenceResult<T> = Result<T, InferenceError>;

#[derive(Debug, thiserror::Error)]
pub enum InferenceError {
    #[error("runtime error")]
    RuntimeError,

    #[error("ONNX error")]
    OnnxError,

    #[error("Invalid encoding")]
    InvalidEncodingError,

    #[error("Corrupt input tensor")]
    CorruptInputTensor,

    #[error("model reshape failed")]
    ShapeError(#[from] ndarray::ShapeError),

    #[error("Bytes to f32 vec conversion failed")]
    BytesToVecConversionError(#[from] std::io::Error),

    #[error("Configuration of model's input type and/or shape failed")]
    CorruptInputTypeOrShape(#[from] tract_onnx::tract_core::anyhow::Error)
}