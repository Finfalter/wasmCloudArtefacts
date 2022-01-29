mod tract;
pub use tract::{TractSession, TractEngine};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{collections::{btree_map::Keys, BTreeMap}};

/// Graph
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Graph(pub u32);

/// GraphEncoding
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct GraphEncoding(pub u8);

impl GraphEncoding {
    pub const GRAPH_ENCODING_OPENVINO: u8 = 0;
    pub const GRAPH_ENCODING_ONNX:     u8 = 1;
}

/// GraphBuilder
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct GraphBuilder(pub Vec<u8>);

/// GraphExecutionContext
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct GraphExecutionContext {
    pub gec: u32,
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
pub struct TensorType(pub u8);

impl TensorType {
    pub const F16: u8 = 0;
    pub const F32: u8 = 1;
    pub const  U8: u8 = 2;
    pub const I32: u8 = 3;
}

#[derive(Default)]
pub struct ModelState {
    pub executions: BTreeMap<GraphExecutionContext, TractSession>,
    pub models: BTreeMap<Graph, Vec<u8>>,
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
    async fn load(&self, builder: &GraphBuilder, encoding: GraphEncoding, target: ExecutionTarget) -> InferenceResult<Graph>;
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
}