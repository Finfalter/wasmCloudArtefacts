use async_trait::async_trait;
use tokio::sync::RwLock;

use std::io::Cursor;

use crate::inference::{
    ExecutionTarget, Graph, GraphEncoding, GraphExecutionContext,
    InferenceEngine, InferenceError, InferenceResult, ModelState};

use tract_onnx::prelude::*;
use tract_onnx::prelude::Tensor as TractTensor;
use tract_onnx::{prelude::Graph as TractGraph, tract_hir::infer::InferenceOp};

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

#[derive(Default, Clone)]
pub struct TractEngine {
    state: Arc<RwLock<ModelState>>
}

#[async_trait]
impl InferenceEngine for TractEngine {
    /// load
    async fn load(
        &self,
        builder: &[u8],
        encoding: &GraphEncoding,
        target: &ExecutionTarget,
    ) -> InferenceResult<Graph> {
        log::info!("load: encoding: {:#?}, target: {:#?}", encoding, target);

        if encoding != &GraphEncoding(GraphEncoding::GRAPH_ENCODING_ONNX) {
            log::error!("load current implementation can only load ONNX models");
            return Err(InferenceError::InvalidEncodingError);
        }
        let model_bytes = builder.to_vec();
        let mut state = self.state.write().await;
        let graph = state.key(state.models.keys());

        log::info!(
            "load: inserting graph: {:#?} with size {:#?}",
            graph,
            model_bytes.len()
        );

        state.models.insert(graph, model_bytes);

        log::info!("load: current number of models: {:#?}", state.models.len());

        Ok(graph)
    }

    async fn init_execution_context(&self, graph: Graph) -> InferenceResult<GraphExecutionContext>
    {
        log::info!("init_execution_context: graph: {:#?}", graph);

        let mut state = self.state.write().await;
        let mut model_bytes = match state.models.get(&graph) {
            Some(mb) => Cursor::new(mb),
            None => {
                log::error!(
                    "init_execution_context: cannot find model in state with graph {:#?}",
                    graph
                );
                return Err(InferenceError::RuntimeError);
            }
        };

        let model = tract_onnx::onnx().model_for_read(&mut model_bytes).unwrap();

        let gec = state.key(state.executions.keys());
        log::info!(
            "init_execution_context: inserting graph execution context: {:#?}",
            gec
        );

        state
            .executions
            .insert(gec, TractSession::with_graph(model));

        Ok(gec)
    }
}