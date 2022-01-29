use async_trait::async_trait;

// use super::engine::InferenceEngine;
// use super::Graph;

use crate::inference::{
    ExecutionTarget, Graph, GraphEncoding, 
    GraphBuilder, InferenceEngine, InferenceResult};

use tract_onnx::prelude::*;
use tract_onnx::prelude::Tensor as TractTensor;
use tract_onnx::{prelude::Graph as TractGraph, tract_hir::infer::InferenceOp};

//use engine::InferenceEngine;
//mod inference::inference_engine;
//use inference::MlState;

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

pub struct TractEngine {}

#[async_trait]
impl InferenceEngine for TractEngine {
    async fn load(
        &self, 
        builder: &GraphBuilder, 
        encoding: GraphEncoding, 
        target: ExecutionTarget
    ) -> InferenceResult<Graph> 
    {
        Ok(Graph(42))
    }
}




// /// load
// async fn load(&self, _ctx: &Context, arg: &LoadInput) -> RpcResult<LoadResult> 
// {
//     debug!("load() - processing request load()");

//     let builder = &arg.builder;
//     let encoding = &arg.encoding;
//     let target = &arg.target;

//     info!("load() - encoding: {:#?}, target: {:#?}", encoding, target);

//     if encoding.encoding != GraphEncoding::GRAPH_ENCODING_ONNX {
//         error!("load() - current implementation can only load ONNX models");

//         let result_with_error = LoadResult {
//             result: catch_error_as(GuestErrorWrap(GuestErrorWrap::InvalidEncodingError)),
//             graph: Graph{graph: std::u32::MAX},
//         };
        
//         return Ok(result_with_error);
//     };

//     let model_bytes = builder.to_vec();

//     let mut state = self.state.write().unwrap();
    
//     let graph_handle = state.key(state.models.keys());
    
//     info!("load() - inserting graph: {:#?} with size {:#?}",
//         graph_handle,
//         model_bytes.len()
//     );

//     state.models.insert(graph_handle, model_bytes);

//     info!("load() - current number of models: {:#?}", state.models.len());

//     let result_ok = LoadResult {
//         result: get_valid_base_result(),
//         graph: Graph::from(graph_handle),
//     };

//     Ok(result_ok)
// }