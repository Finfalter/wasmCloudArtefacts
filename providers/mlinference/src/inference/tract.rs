use async_trait::async_trait;
use tokio::sync::RwLock;

use std::io::Cursor;

use crate::inference::{
    ExecutionTarget, Graph, GraphEncoding, GraphExecutionContext,
    InferenceEngine, InferenceError, InferenceResult, ModelState};

use wasmcloud_interface_mlinference::{ Tensor, TensorType, InferenceOutput, ResultStatus };

use byteorder::{LittleEndian, ReadBytesExt};

use ndarray::Array;
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
    ) -> InferenceResult<Graph> 
    {
        log::info!("==============> load() - encoding: {:#?}, target: {:#?}", encoding, target);

        if encoding != &GraphEncoding(GraphEncoding::GRAPH_ENCODING_ONNX) 
        {
            log::error!("load current implementation can only load ONNX models");
            return Err(InferenceError::InvalidEncodingError);
        }
        let model_bytes = builder.to_vec();
        let mut state = self.state.write().await;
        let graph = state.key(state.models.keys());

        log::info!(
            "load() - inserting graph: {:#?} with size {:#?}",
            graph,
            model_bytes.len()
        );

        state.models.insert(graph, model_bytes);

        log::info!("load() - current number of models: {:#?} ==============>", state.models.len());

        Ok(graph)
    }

    async fn init_execution_context(&self, graph: Graph) -> InferenceResult<GraphExecutionContext>
    {
        log::info!("==============> init_execution_context() - graph: {:#?}", graph);

        let mut state = self.state.write().await;
        let mut model_bytes = match state.models.get(&graph) 
        {
            Some(mb) => Cursor::new(mb),
            None => {
                log::error!(
                    "init_execution_context() - cannot find model in state with graph {:#?}",
                    graph
                );
                return Err(InferenceError::RuntimeError);
            }
        };

        let model = tract_onnx::onnx().model_for_read(&mut model_bytes).unwrap();

        let gec = state.key(state.executions.keys());
        log::info!(
            "init_execution_context() - inserting graph execution context: {:#?}",
            gec
        );

        state
            .executions
            .insert(gec, TractSession::with_graph(model));

        log::info!("init_execution_context() ==============>");

        Ok(gec)
    }

    /// set_input
    async fn set_input(&self, context: GraphExecutionContext, index: u32, tensor: &Tensor) -> InferenceResult<()> 
    {
        log::debug!("==============> set_input()");
        
        let mut state = self.state.write().await;
        let execution = match state.executions.get_mut(&context) {
            Some(s) => s,
            None => {
                log::error!(
                    "set_input() - cannot find session in state with context {:#?}",
                    context
                );

                return Err(InferenceError::RuntimeError);
            }
        };

        let shape = tensor
            .dimensions
            .iter()
            .map(|d| *d as usize)
            .collect::<Vec<_>>();

        execution.graph.set_input_fact(
            index as usize,
            InferenceFact::dt_shape(f32::datum_type(), shape.clone()))?;
        
        let data: Vec<f32> = bytes_to_f32_vec(tensor.data.as_slice().to_vec())?;
        let input: TractTensor = Array::from_shape_vec(shape, data)?.into();

        match execution.input_tensors {
            Some(ref mut input_arrays) => {
                input_arrays.push(input);
                log::info!(
                    "set_input() - input arrays now contains {} items",
                    input_arrays.len(),
                );
            }
            None => {
                execution.input_tensors = Some(vec![input]);
            }
        };
        log::debug!("set_input() ==============> ");
        Ok(())
    }

    /// compute()
    async fn compute(&self, context: GraphExecutionContext) -> InferenceResult<()> 
    {
        log::debug!("==============> compute()");
        
        let mut state = self.state.write().await;
        let mut execution = match state.executions.get_mut(&context) {
            Some(s) => s,
            None => {
                log::error!(
                    "compute() - cannot find session in state with context {:#?}",
                    context
                );

                return Err(InferenceError::RuntimeError);
            }
        };

        // TODO
        //
        // There are two `.clone()` calls here that could prove
        // to be *very* inneficient, one in getting the input tensors,
        // the other in making the model runnable.
        let input_tensors: Vec<TractTensor> = execution
            .input_tensors
            .as_ref()
            .unwrap_or(&vec![])
            .clone()
            .into_iter()
            .collect();

        log::info!(
            "compute() - input tensors contains {} elements",
            input_tensors.len()
        );

        // Some ONNX models don't specify their input tensor
        // shapes completely, so we can only call `.into_optimized()` after we
        // have set the input tensor shapes.
        let output_tensors = execution
            .graph
            .clone()
            .into_optimized()?
            .into_runnable()?
            .run(input_tensors.into())?;

        log::info!(
            "compute() - output tensors contains {} elements",
            output_tensors.len()
        );
        match execution.output_tensors {
            Some(_) => {
                log::error!("compute: existing data in output_tensors, aborting");
                return Err(InferenceError::RuntimeError);
            }
            None => {
                execution.output_tensors = Some(output_tensors.into_iter().collect());
            }
        };
        log::debug!("compute() ==============> ");
        Ok(())
    }

    /// get_output
    async fn get_output(
        &self,
        context: GraphExecutionContext,
        index: u32
    ) -> InferenceResult<InferenceOutput> 
    {
        log::debug!("==============> get_output()");
        let state = self.state.read().await;
        let execution = match state.executions.get(&context) {
            Some(s) => s,
            None => {
                log::error!(
                    "compute() - cannot find session in state with context {:#?}",
                    context
                );

                return Err(InferenceError::RuntimeError);
            }
        };

        let output_tensors = match execution.output_tensors {
            Some(ref oa) => oa,
            None         => {
                log::error!("get_output() - output_tensors for session is none. Perhaps you haven't called compute yet?");
                return Err(InferenceError::RuntimeError);
            }
        };

        let tensor = match output_tensors.get(index as usize) {
            Some(a) => a,
            None    => {
                log::error!(
                    "get_output() - output_tensors does not contain index {}",
                    index
                );
                return Err(InferenceError::RuntimeError);
            }
        };

        let bytes = f32_vec_to_bytes(tensor.as_slice().unwrap().to_vec());

        let io = InferenceOutput {
            result: ResultStatus { has_error: false, error: None },
            tensor: Tensor {
                ttype: TensorType{ ttype: 0},
                dimensions: vec![],
                data: bytes
            }
        };
        log::debug!("get_output() ==============>");
        Ok(io)
    }

    /// remove model state
    async fn drop_model_state(&self, graph: &Graph, gec: &GraphExecutionContext) 
    {
        let mut state = self.state.write().await;

        state.models.remove(graph);
        state.executions.remove(gec);
    }
}

pub type Result<T> = std::io::Result<T>;

pub fn bytes_to_f32_vec(data: Vec<u8>) -> Result<Vec<f32>> {
    //let chunks: Vec<&[u8]> = data.chunks(4).collect();
    let chunks = data.chunks(4);
    //let v: Vec<Result<f32>> = chunks
    let v = chunks
        .into_iter()
        .map(|c| {
            let mut rdr = Cursor::new(c);
            Ok(rdr.read_f32::<LittleEndian>()?)
        });
        //.collect();

    v.collect()
}

pub fn f32_vec_to_bytes(data: Vec<f32>) -> Vec<u8> {
    let sum: f32 = data.iter().sum();
    log::info!(
        "f32_vec_to_bytes() - flatten output tensor contains {} elements with sum {}",
        data.len(),
        sum
    );
    let chunks: Vec<[u8; 4]> = data.into_iter().map(|f| f.to_le_bytes()).collect();
    let result: Vec<u8> = chunks.iter().flatten().copied().collect();

    log::info!(
        "f32_vec_to_bytes() - flatten byte output tensor contains {} elements",
        result.len()
    );
    result
}