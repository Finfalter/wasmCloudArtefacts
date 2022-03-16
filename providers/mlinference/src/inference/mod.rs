mod tract;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{btree_map::Keys, BTreeMap};
pub use tract::{bytes_to_f32_vec, f32_vec_to_bytes, TractEngine, TractSession};
use wasmcloud_interface_mlinference::{InferenceOutput, Tensor, TensorType};

/// Graph (model number)
pub type Graph = u32;

/// GraphEncoding
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphEncoding {
    Onnx,
    OpenVino,
    Tensorflow,
}

impl Default for GraphEncoding {
    fn default() -> GraphEncoding {
        GraphEncoding::Onnx
    }
}

/// GraphExecutionContext
pub type GraphExecutionContext = u32;

/// ExecutionTarget
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionTarget {
    Cpu,
    Gpu,
    Tpu,
}
impl Default for ExecutionTarget {
    fn default() -> Self {
        ExecutionTarget::Cpu
    }
}

/// TensorType
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct TType(pub u8);

impl TType {
    pub const F16: u8 = 0;
    pub const F32: u8 = 1;
    pub const U8: u8 = 2;
    pub const I32: u8 = 3;
}

impl From<TensorType> for TType {
    fn from(tt: TensorType) -> TType {
        TType(tt.ttype)
    }
}

impl From<TType> for TensorType {
    fn from(tt: TType) -> TensorType {
        TensorType { ttype: tt.0 }
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
    async fn load(
        &self,
        builder: &[u8],
        encoding: &GraphEncoding,
        target: &ExecutionTarget,
    ) -> InferenceResult<Graph>;
    async fn init_execution_context(&self, graph: Graph) -> InferenceResult<GraphExecutionContext>;
    async fn set_input(
        &self,
        context: GraphExecutionContext,
        index: u32,
        tensor: &Tensor,
    ) -> InferenceResult<()>;
    async fn compute(&self, context: GraphExecutionContext) -> InferenceResult<()>;
    async fn get_output(
        &self,
        context: GraphExecutionContext,
        index: u32,
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
    CorruptInputTypeOrShape(#[from] tract_onnx::tract_core::anyhow::Error),
}
