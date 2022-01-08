#![allow(dead_code)]

use std::{
    collections::{btree_map::Keys, BTreeMap},
    cmp::Ordering,
    sync::Arc,
};
use serde::{Deserialize, Serialize};
use wasmcloud_interface_mlinference::{BaseResult, Graph, GuestError, RuntimeError, GraphExecutionContext};

use tract_onnx::prelude::*;
use tract_onnx::prelude::Tensor as TractTensor;
use tract_onnx::{prelude::Graph as TractGraph, tract_hir::infer::InferenceOp};

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;

#[derive(Debug, thiserror::Error)]
pub enum MlError {
    #[error("guest error")]
    RuntimeErrorWrap(#[from] RuntimeErrorWrap),

    #[error("other error")]
    GuestErrorWrap(GuestErrorWrap),
}

pub fn get_valid_base_result() -> BaseResult {
    BaseResult {
        has_error: false,
        runtime_error: None,
        guest_error: None
    }
}

pub fn catch_error_as(error: MlError) -> BaseResult {
    let mut re: Option<RuntimeError> = None;
    let mut ge: Option<GuestError> = None;

    match error {
        MlError::RuntimeErrorWrap(rew) => { re = Some(RuntimeError::from(rew)); },
        MlError::GuestErrorWrap(gew)     => { ge = Some(GuestError::from(gew)); },
    }

    BaseResult {
        has_error: true,
        runtime_error: re,
        guest_error: ge,
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum RuntimeErrorWrap {
    #[error("Invalid flag value")]
    RuntimeError = 0,
    #[error("Invalid flag value")]
    OpenVinoError = 1,
    #[error("Invalid flag value")]
    OnnxError = 2,
    #[error("Invalid flag value")]
    ContextNotFound = 3,
}

impl From<RuntimeErrorWrap> for RuntimeError {
    fn from(rew: RuntimeErrorWrap) -> RuntimeError {
        match rew {
            RuntimeErrorWrap::RuntimeError => RuntimeError{runtime_error: 0},
            RuntimeErrorWrap::OpenVinoError => RuntimeError{runtime_error: 1},
            RuntimeErrorWrap::OnnxError => RuntimeError{runtime_error: 2},
            RuntimeErrorWrap::ContextNotFound => RuntimeError{runtime_error: 3},
        }
    }
}

#[derive(Debug)]
pub enum GuestErrorWrap {
    ModelError = 0,
    InvalidEncodingError = 1,
    CorruptInputTensor = 2,
}

impl From<GuestErrorWrap> for GuestError {
    fn from(gew: GuestErrorWrap) -> GuestError {
        match gew {
            GuestErrorWrap::ModelError => GuestError{model_error: 0},
            GuestErrorWrap::InvalidEncodingError => GuestError{model_error: 1},
            GuestErrorWrap::CorruptInputTensor => GuestError{model_error: 2},
        }
    }
}

#[non_exhaustive]
#[derive(Copy, Clone)]
pub struct GraphEncoding;

#[allow(dead_code)]
impl GraphEncoding {
    pub const GRAPH_ENCODING_OPENVINO: u8 = 0;
    pub const GRAPH_ENCODING_ONNX:     u8 = 1;
}

#[non_exhaustive]
#[derive(Copy, Clone)]
pub struct ExecutionTarget;

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

#[derive(Copy, Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
 pub struct GECWrap(u32);

 impl From<u32> for GECWrap {
    fn from(e: u32) -> GECWrap {
        GECWrap(e)
    }
}

impl From<GECWrap> for u32 {
    fn from(gec: GECWrap) -> u32 {
        gec.0
    }
}

impl PartialOrd for GECWrap {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GECWrap {
    fn cmp(&self, other: &Self) -> Ordering {
        let (s, o) = (*self, *other);
        let s: u32 = s.into();
        let o: u32 = o.into();
        s.cmp(&o)
    }
}

impl From<GECWrap> for GraphExecutionContext {
    fn from(gec: GECWrap) -> GraphExecutionContext {
        GraphExecutionContext{gec: gec.0}
    }
}

#[derive(Default)]
pub struct State {
    pub executions: BTreeMap<GECWrap, TractSession>,
    pub models: BTreeMap<GraphWrap, Vec<u8>>,
}

impl State {
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

#[derive(Debug)]
pub struct TractSession {
    pub graph: TractGraph<InferenceFact, Box<dyn InferenceOp>>,
    pub input_tensors: Option<Vec<TractTensor>>,
    pub output_tensors: Option<Vec<Arc<TractTensor>>>,
}

impl TractSession {
    pub fn with_graph(graph: TractGraph<InferenceFact, Box<dyn InferenceOp>>) -> Self {
        Self {
            graph,
            input_tensors: None,
            output_tensors: None,
        }
    }
}

pub type Result<T> = std::io::Result<T>;

pub fn bytes_to_f32_vec(data: Vec<u8>) -> Result<Vec<f32>> {
    let chunks: Vec<&[u8]> = data.chunks(4).collect();
    let v: Vec<Result<f32>> = chunks
        .into_iter()
        .map(|c| {
            let mut rdr = Cursor::new(c);
            Ok(rdr.read_f32::<LittleEndian>()?)
        })
        .collect();

    v.into_iter().collect()
}

pub fn f32_vec_to_bytes(data: Vec<f32>) -> Vec<u8> {
    let chunks: Vec<[u8; 4]> = data.into_iter().map(|f| f.to_le_bytes()).collect();
    let mut result: Vec<u8> = Vec::new();

    // TODO
    // simplify this to potentially a single map.
    for c in chunks {
        for u in c.iter() {
            result.push(*u);
        }
    }
    result
}