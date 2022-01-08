//! mlinference capability provider
//!
mod lib;

use lib::{GraphWrap, GECWrap, GraphEncoding, GuestErrorWrap, RuntimeErrorWrap, 
    TractSession, State, f32_vec_to_bytes, bytes_to_f32_vec, catch_error_as, get_valid_base_result, MlError::*};

use std::{
    sync::{Arc, RwLock},
    io::Cursor,
};
use log::{debug, info, error};
use wasmbus_rpc::provider::prelude::*;
use wasmcloud_interface_mlinference::{
    Mlinference, MlinferenceReceiver, LoadInput, Graph, LoadResult, GraphExecutionContext, IecResult, 
    SetInputStruct, BaseResult, GetOutputStruct, InferenceResult
};

use ndarray::Array;
use tract_onnx::prelude::Tensor as TractTensor;
use tract_onnx::prelude::*;

// main (via provider_main) initializes the threaded tokio executor,
// listens to lattice rpcs, handles actor links,
// and returns only when it receives a shutdown message
//
fn main() -> Result<(), Box<dyn std::error::Error>> {
    provider_main(MlinferenceProvider::default())?;

    eprintln!("mlinference provider exiting");
    Ok(())
}

/// mlinference capability provider implementation
#[derive(Default, Clone, Provider)]
#[services(Mlinference)]
struct MlinferenceProvider {
    pub state: Arc<RwLock<State>>,
}

/// use default implementations of provider message handlers
impl ProviderDispatch for MlinferenceProvider {}
impl ProviderHandler for MlinferenceProvider {}

/// Handle Mlinference methods
#[async_trait]
impl Mlinference for MlinferenceProvider {
    /// load
    async fn load(&self, _ctx: &Context, arg: &LoadInput) -> RpcResult<LoadResult> 
    {
        debug!("load() - processing request load()");

        let builder = &arg.builder;
        let encoding = &arg.encoding;
        let target = &arg.target;

        info!("load() - encoding: {:#?}, target: {:#?}", encoding, target);

        if encoding.encoding != GraphEncoding::GRAPH_ENCODING_ONNX {
            error!("load() - current implementation can only load ONNX models");

            let result_with_error = LoadResult {
                result: catch_error_as(GuestErrorWrap(GuestErrorWrap::InvalidEncodingError)),
                graph: Graph{graph: std::u32::MAX},
            };
            
            return Ok(result_with_error);
        };

        let model_bytes = builder.to_vec();

        let mut state = self.state.write().unwrap();
        
        let graph_handle = state.key(state.models.keys());
        
        info!("load() - inserting graph: {:#?} with size {:#?}",
            graph_handle,
            model_bytes.len()
        );

        state.models.insert(graph_handle, model_bytes);

        info!("load() - current number of models: {:#?}", state.models.len());

        let result_ok = LoadResult {
            result: get_valid_base_result(),
            graph: Graph::from(graph_handle),
        };

        Ok(result_ok)
    }

    /// init_execution_context
    async fn init_execution_context(
        &self,
        _ctx: &Context,
        arg: &Graph,
    ) -> RpcResult<IecResult> 
    {
        let graph = GraphWrap::from(arg.graph);

        info!("init_execution_context: graph: {:#?}", graph);

        // TOD0: should not panic in case the lock is not available
        let mut state = self.state.write().unwrap();
        let mut model_bytes = match state.models.get(&graph) 
        {
            Some(mb) => Cursor::new(mb),

            None => {
                error!("init_execution_context: cannot find model in state with graph {:#?}", graph);

                let result_with_error = IecResult {
                    result: catch_error_as(RuntimeErrorWrap(RuntimeErrorWrap::RuntimeError)),
                    gec: GraphExecutionContext{gec: std::u32::MAX},
                };
                
                return Ok(result_with_error);
            }
        };

        let model = match tract_onnx::onnx().model_for_read(&mut model_bytes) 
        {
            Ok (v) => v,
        
            Err(e)=> {
                error!("init_execution_context() - problems with reading given model: {:#?}", e);

                let result_with_error = IecResult {
                    result: catch_error_as(GuestErrorWrap(GuestErrorWrap::ModelError)),
                    gec: GraphExecutionContext{gec: std::u32::MAX},
                };
                return Ok::<IecResult, wasmbus_rpc::RpcError>(result_with_error);
            }
        };

        let gec = state.key(state.executions.keys());
        info!("init_execution_context: inserting graph execution context: {:#?}", gec);

        state
            .executions
            .insert(gec, TractSession::with_graph(model));

        let result_ok = IecResult {
            result: get_valid_base_result(),
            gec: GraphExecutionContext::from(gec),
        };

        Ok(result_ok)
    }

    /// SetInput
    /// If there are multiple input tensors, the guest
    /// should call this function in order, as this actually
    /// constructs the final input tensor used for the inference.
    /// If we wanted to avoid this, we could create an intermediary
    /// HashMap<u32, Array<TIn, D>> and collapse it into a Vec<Array<TIn, D>>
    /// when performing the inference.
    async fn set_input(&self, _ctx: &Context, arg: &SetInputStruct) -> RpcResult<BaseResult> 
    {
        let mut state = self.state.write().unwrap();

        let gec_wrap = GECWrap::from(arg.context.gec);
        let index: u32 = match arg.index {
            Some(v) => v,
            None => 0,
        };
        let tensor = &arg.tensor;

        let execution = match state.executions.get_mut(&gec_wrap) 
        {
            Some(s) => s,
            None => {
                error!("set_input() - cannot find session in state with context {:#?}", gec_wrap);
                return Ok(catch_error_as(RuntimeErrorWrap(RuntimeErrorWrap::ContextNotFound)));
            }
        };

        let shape = tensor
        .dimensions
        .iter()
        .map(|d| *d as usize)
        .collect::<Vec<_>>();

        match execution.graph.set_input_fact(
        index as usize,
        InferenceFact::dt_shape(f32::datum_type(), shape.clone()),) 
        {
            Ok(s) => s,

            Err(e) => {
                error!("set_input() - cannot set input fact {:#?}", e);
                return Ok(catch_error_as(RuntimeErrorWrap(RuntimeErrorWrap::RuntimeError)));
            }
        };
        
        let data = bytes_to_f32_vec(tensor.data.as_slice().to_vec())?;
        let input: TractTensor = match Array::from_shape_vec(shape, data) 
        {
            Ok(s) => s.into(),

            Err(e) => {
                error!("set_input() - corrupt tensor input {:#?}", e);
                return Ok(catch_error_as(GuestErrorWrap(GuestErrorWrap::CorruptInputTensor)));
            }
        };

        match execution.input_tensors 
        {
            Some(ref mut input_arrays) => {
                input_arrays.push(input);
                log::info!(
                    "set_input: input arrays now contains {} items",
                    input_arrays.len(),
                );
            }
            None => {
                execution.input_tensors = Some(vec![input]);
            }
        };

        Ok(get_valid_base_result())
    }

    async fn compute(&self, _ctx: &Context, arg: &GraphExecutionContext) -> RpcResult<BaseResult> 
    {
        let mut state = self.state.write().unwrap();

        let gec_wrap = GECWrap::from(arg.gec);

        let execution = match state.executions.get_mut(&gec_wrap) 
        {
            Some(s) => s,

            None => {
                error!("set_input() - cannot find session in state with context {:#?}", gec_wrap);
                return Ok(catch_error_as(RuntimeErrorWrap(RuntimeErrorWrap::ContextNotFound)));
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

        info!("compute() - input tensors contains {} elements", input_tensors.len() );

        // Some ONNX models don't specify their input tensor
        // shapes completely, so we can only call `.into_optimized()` after we
        // have set the input tensor shapes.
        let output_tensors = execution
            .graph
            .clone()
            .into_optimized().map_err(|_| catch_error_as(RuntimeErrorWrap(RuntimeErrorWrap::OnnxError))).unwrap()
            .into_runnable().map_err(|_| catch_error_as(RuntimeErrorWrap(RuntimeErrorWrap::OnnxError))).unwrap()
            .run(input_tensors.into()).map_err(|_| catch_error_as(RuntimeErrorWrap(RuntimeErrorWrap::OnnxError))).unwrap();

        info!("compute() - output tensor contains {} elements", output_tensors.len() );

        match execution.output_tensors 
        {
            Some(_) => {
                error!("compute() - existing data in output_tensors, aborting");
                return Ok(catch_error_as(RuntimeErrorWrap(RuntimeErrorWrap::RuntimeError)));
            },

            None => {
                execution.output_tensors = Some(output_tensors.into_iter().collect());
            }
        };

        Ok(get_valid_base_result())
    }

    /// get_output
    async fn get_output(&self, _ctx: &Context, arg: &GetOutputStruct) -> RpcResult<InferenceResult> 
    {
        let index = match arg.index {
            Some(val) => val,
            None => 0
        };
        
        let state = self.state.read().unwrap();

        let gec_wrap = GECWrap::from(arg.gec.gec);

        let execution = match state.executions.get(&gec_wrap) 
        {
            Some(s) => s,

            None => {
                error!("set_input() - cannot find session in state with context {:#?}", gec_wrap);

                let result_with_error = InferenceResult {
                    result: catch_error_as(RuntimeErrorWrap(RuntimeErrorWrap::RuntimeError)),
                    buffer: vec![],
                    size: 0,
                };

                return Ok(result_with_error);
            }
        };

        let output_tensors = match execution.output_tensors 
        {
            Some(ref oa) => oa,

            None => {
                error!("get_output() - output_tensors for session is none. Perhaps compute() was not called, yet.");
                
                let result_with_error = InferenceResult {
                    result: catch_error_as(RuntimeErrorWrap(RuntimeErrorWrap::RuntimeError)),
                    buffer: vec![],
                    size: 0,
                };

                return Ok(result_with_error);
            }
        };

        let tensor = match output_tensors.get(index as usize) {
            Some(a) => a,

            None => {
                error!("get_output() - output_tensors does not contain index {}", index);
                
                let result_with_error = InferenceResult {
                    result: catch_error_as(RuntimeErrorWrap(RuntimeErrorWrap::RuntimeError)),
                    buffer: vec![],
                    size: 0,
                };

                return Ok(result_with_error);
            }
        };

        let bytes = f32_vec_to_bytes(tensor.as_slice().unwrap().to_vec());
        let size = bytes.len();

        let ir = InferenceResult {
            result: get_valid_base_result(),
            buffer: bytes,
            size: size as u64
        };

        Ok(ir)
    }
}