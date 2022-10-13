use crate::inference::{
    ExecutionTarget, Graph, GraphEncoding, GraphExecutionContext, InferenceEngine, InferenceError,
    InferenceResult,
};
use async_trait::async_trait;
#[cfg(feature = "edgetpu")]
use edgetpu::EdgeTpuContext;
#[cfg(any(feature = "tflite", feature = "edgetpu"))]
use tflite::{op_resolver::OpResolver, ops::builtin::BuiltinOpResolver, Interpreter, FlatBufferModel, InterpreterBuilder};
use std::collections::{btree_map::Keys, BTreeMap};
use std::sync::Arc;
use tokio::sync::RwLock;
use wasmcloud_interface_mlinference::{
    InferenceOutput, Status, Tensor, ValueType, TENSOR_FLAG_ROW_MAJOR,
};

#[derive(Default, Clone)]
pub struct TfLiteEngine<'a> {
    state: Arc<RwLock<ModelState<'a>>>,
}

#[derive(Default)]
pub struct ModelState<'a> {
    executions: BTreeMap<GraphExecutionContext, TfLiteSession<'a, BuiltinOpResolver>>,
    models: BTreeMap<Graph, Vec<u8>>,
}

impl<'a> ModelState<'a> {
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

pub struct TfLiteSession<'a, BuiltinOpResolver: OpResolver> {
    pub graph: Interpreter<'a, BuiltinOpResolver>,
    pub encoding: GraphEncoding,
    pub input_tensors: usize,
    pub output_tensors: Option<Vec<Tensor>>,
    #[cfg(feature = "edgetpu")]
    pub edgetpu_context: Option<edgetpu::EdgeTpuContext>,
}

impl<'a> TfLiteSession<'a, BuiltinOpResolver> {
    pub fn with_graph(
        graph: Interpreter<'a, BuiltinOpResolver>,
        encoding: GraphEncoding,
        #[cfg(feature = "edgetpu")]
        edgetpu_context: Option<edgetpu::EdgeTpuContext>,
    ) -> Self {
        Self {
            graph,
            encoding,
            #[cfg(feature = "edgetpu")]
            edgetpu_context,
            input_tensors: 0,
            output_tensors: None,
        }
    }
}

#[async_trait]
impl<'a> InferenceEngine for TfLiteEngine<'a> {
    /// load
    async fn load(&self, model: &[u8]) -> InferenceResult<Graph> {
        let model_bytes = model.to_vec();

        let mut state = self.state.write().await;
        let graph = state.key(state.models.keys());

        log::debug!(
            "load() - inserting graph: {:#?} with size {:#?}",
            graph,
            model_bytes.len()
        );

        state.models.insert(graph, model_bytes);

        log::debug!(
            "load() - current number of models: {:#?}",
            state.models.len()
        );

        Ok(graph)
    }

    /// init execution context
    async fn init_execution_context(
        &self,
        graph: Graph,
        target: &ExecutionTarget,
        encoding: &GraphEncoding,
    ) -> InferenceResult<GraphExecutionContext> {
        log::debug!("init_execution_context() - entering");

        log::debug!(
            "init_execution_context() - detected execution target: {:?}",
            target
        );

        log::debug!(
            "init_execution_context() - detected encoding: {:?}",
            encoding
        );

        #[cfg(feature = "edgetpu")]
        if !matches!(target, &ExecutionTarget::Tpu) && !matches!(target, &ExecutionTarget::Cpu) {
            log::error!(
                "TfLiteEngine does not support execution target '{:?}'",
                target
            );
            return Err(InferenceError::UnsupportedExecutionTarget);
        }

        let mut state = self.state.write().await;
        let model_bytes = match state.models.get(&graph) {
            Some(mb) => mb,
            None => {
                log::error!(
                    "init_execution_context() - cannot find model in state with graph {:#?}",
                    graph
                );
                return Err(InferenceError::RuntimeError);
            }
        };

        let model: FlatBufferModel = match encoding {
            GraphEncoding::TfLite => FlatBufferModel::build_from_buffer(model_bytes.to_vec())
                .map_err(|_| {
                    log::error!(
                        "init_execution_context() - building FlatBufferModel from buffer failed"
                    );
                    InferenceError::FailedToBuildModelFromBuffer
                })?,

            _ => {
                log::error!(
                    "init_execution_context() - requested encoding '{:?}' is currently not supported",
                    encoding
                );
                return Err(InferenceError::InvalidEncodingError);
            }
        };

        let resolver = BuiltinOpResolver::default();

        #[cfg(feature = "edgetpu")]
        if matches!(target, &ExecutionTarget::Tpu) {
            resolver.add_custom(edgetpu::custom_op(), edgetpu::register_custom_op());
        }

        let builder = InterpreterBuilder::new(model, resolver).map_err(|_| {
            log::error!("init_execution_context() - failed to get InterpreterBuilder");
            InferenceError::InterpreterBuilderError
        })?;

        let mut interpreter = builder.build().map_err(|_| {
            log::error!("init_execution_context() - failed building Interpreter");
            InferenceError::InterpreterBuildError
        })?;

        #[cfg(feature = "edgetpu")]        
        let mut edgetpu_context: Option<edgetpu::EdgeTpuContext> = None;

        #[cfg(feature = "edgetpu")]
        if matches!(target, &ExecutionTarget::Tpu) {
            edgetpu_context = Some(EdgeTpuContext::open_device().map_err(|_| {
                log::error!("init_execution_context() - failed to get edge TPU context");
                InferenceError::FailedToBuildModelFromBuffer
            })?);

            interpreter.set_external_context(
                tflite::ExternalContextType::EdgeTpu,
                // https://users.rust-lang.org/t/calling-method-of-some-in-a-borrowed-option-without-moving-it/3203
                edgetpu_context.as_mut().unwrap().to_external_context(),
            );

            interpreter.set_num_threads(1);
        }

        interpreter.allocate_tensors().map_err(|_| {
            log::error!("init_execution_context() - Interpreter: tensor allocation failed");
            InferenceError::TensorAllocationError
        })?;

        let gec = state.key(state.executions.keys());

        log::debug!(
            "init_execution_context() - inserting graph execution context: {:#?}",
            gec
        );

        state.executions.insert(
            gec,
            TfLiteSession::with_graph(
                interpreter, 
                encoding.to_owned(), 
                #[cfg(feature = "edgetpu")]                
                edgetpu_context
            ),
        );

        log::debug!("init_execution_context() - passed");

        Ok(gec)
    }

    /// set_input
    async fn set_input(
        &self,
        context: GraphExecutionContext,
        index: u32,
        tensor: &Tensor,
    ) -> InferenceResult<()> {
        log::debug!(
            "entering set_input() - with context: {:?}, index: {}, tensor: {:?}",
            &context,
            index,
            tensor
        );

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

        let tensor_index = execution.graph.inputs()[execution.input_tensors];

        // prepare for multiple input tensors
        execution.input_tensors = execution.input_tensors + 1;

        log::debug!(
            "set_input() - required shape: {:?}",
            execution.graph.tensor_info(tensor_index).unwrap().dims,
        );

        execution
            .graph
            .tensor_data_mut(tensor_index)
            .unwrap()
            .copy_from_slice(tensor.data.as_slice());

        log::debug!("set_input() - passed");

        Ok(())
    }

    /// compute()
    async fn compute(&self, context: GraphExecutionContext) -> InferenceResult<()> {
        log::debug!("compute() - entering");

        let mut state = self.state.write().await;
        let execution = match state.executions.get_mut(&context) {
            Some(s) => s,
            None => {
                log::error!(
                    "compute() - cannot find session in state with context {:#?}",
                    context
                );
                return Err(InferenceError::RuntimeError);
            }
        };

        let interpreter = &mut execution.graph;

        interpreter.invoke().map_err(|_| {
            log::error!("init_execution_context() - interpreter invokation failed");
            InferenceError::InterpreterInvocationError
        })?;

        let output_tensors = interpreter.outputs();

        log::debug!(
            "compute() - output tensors contains {} elements",
            output_tensors.len()
        );

        let mut result_tensors: Vec<Tensor> = Vec::new();

        for &output in output_tensors {
            let mut results = Vec::new();
            let tensor_info = interpreter.tensor_info(output).ok_or_else(|| {
                log::error!("compute() - info for output tensor could not be evaluated");
                return InferenceError::RuntimeError;
            })?;

            match tensor_info.element_kind {
                tflite::context::ElementKind::kTfLiteUInt8 => {
                    let out_tensor: &[u8] = interpreter.tensor_data(output).map_err(|_| {
                        log::error!("compute() - failed to get output tensor");
                        InferenceError::FailedToBuildModelFromBuffer
                    })?;
                    let scale = tensor_info.params.scale;
                    let zero_point = tensor_info.params.zero_point;
                    results = out_tensor
                        .into_iter()
                        .map(|&x| scale * (((x as i32) - zero_point) as f32))
                        .collect();
                }
                tflite::context::ElementKind::kTfLiteFloat32 => {
                    let out_tensor: &[f32] = interpreter.tensor_data(output).map_err(|_| {
                        log::error!("compute() - failed to get output tensor");
                        InferenceError::FailedToBuildModelFromBuffer
                    })?;
                    results = out_tensor.into_iter().copied().collect();
                }
                _ => eprintln!(
                    "Tensor {} has unsupported output type {:?}.",
                    tensor_info.name, tensor_info.element_kind,
                ),
            }

            let bytes = f32_vec_to_bytes(results).await;

            let result_tensor = Tensor {
                value_types: vec![ValueType::ValueF32],
                dimensions: tensor_info.dims.into_iter().map(|i| i as u32).collect(),
                flags: TENSOR_FLAG_ROW_MAJOR,
                data: bytes,
            };

            result_tensors.push(result_tensor);
        }

        execution.input_tensors = 0;
        execution.output_tensors.replace(result_tensors);

        Ok(())
    }

    /// get_output
    async fn get_output(
        &self,
        context: GraphExecutionContext,
        index: u32,
    ) -> InferenceResult<InferenceOutput> {
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
            None => {
                log::error!(
                    "get_output() - output_tensors for session is none. 
                    Perhaps you haven't called compute yet?"
                );
                return Err(InferenceError::RuntimeError);
            }
        };

        let tensor = match output_tensors.get(index as usize) {
            Some(a) => a,
            None => {
                log::error!(
                    "get_output() - output_tensors does not contain index {}",
                    index
                );
                return Err(InferenceError::RuntimeError);
            }
        };

        let io = InferenceOutput {
            result: Status::Success,
            tensor: tensor.to_owned(),
        };
        Ok(io)
    }

    /// remove model state
    async fn drop_model_state(&self, graph: &Graph, gec: &GraphExecutionContext) {
        let mut state = self.state.write().await;

        state.models.remove(graph);
        state.executions.remove(gec);
    }
}

pub async fn f32_vec_to_bytes(data: Vec<f32>) -> Vec<u8> {
    let sum: f32 = data.iter().sum();
    log::debug!(
        "f32_vec_to_bytes() - flattened output tensor contains {} elements with sum {}",
        data.len(),
        sum
    );
    let chunks: Vec<[u8; 4]> = data.into_iter().map(|f| f.to_le_bytes()).collect();
    let result: Vec<u8> = chunks.iter().flatten().copied().collect();

    log::debug!(
        "f32_vec_to_bytes() - flattened byte output tensor contains {} elements",
        result.len()
    );
    result
}
