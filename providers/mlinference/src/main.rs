//! mlinference capability provider
//!
mod utils;

use utils::{GraphWrap, GECWrap, GraphEncoding, GuestErrorWrap, RuntimeErrorWrap, 
    TractSession, State, bytes_to_f32_vec};

use std::{
    sync::{Arc, RwLock},
    io::Cursor,
};
use log::{debug, info, error};
use wasmbus_rpc::provider::prelude::*;
use wasmcloud_interface_mlinference::{
    Mlinference, MlinferenceReceiver, LoadInput, Graph, LoadResult, GuestError, 
    RuntimeError, GraphExecutionContext, IecResult, SetInputStruct, SetInputResult
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

// #[derive(Default)]
// pub struct State {
//     pub executions: BTreeMap<GraphExecutionContext, TractSession>,
//     pub models: BTreeMap<GraphWrap, Vec<u8>>,
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
    /// accepts a number and calculates its factorial
    async fn calculate(&self, _ctx: &Context, req: &u32) -> RpcResult<u64> {
        debug!("processing request calculate({})", *req);
        Ok(n_factorial(*req))
    }

    /// load
    async fn load(&self, _ctx: &Context, arg: &LoadInput) -> RpcResult<LoadResult> {
        debug!("load() - processing request load()");

        let builder = &arg.builder;
        let encoding = &arg.encoding;
        let target = &arg.target;

        info!("load() - encoding: {:#?}, target: {:#?}", encoding, target);

        if encoding.encoding != GraphEncoding::ONNX {
            error!("load() - current implementation can only load ONNX models");

            let result_with_error = LoadResult {
                has_error: true,
                runtime_error: None,
                guest_error: Some(GuestError::from(GuestErrorWrap::InvalidEncodingError)), 
                graph: Graph{graph: std::u32::MAX},
            };
            
            return Ok(result_with_error);
        };

        let model_bytes = builder.to_vec();

        // TOD0: should not panic in case the lock is not available
        let mut state = self.state.write().unwrap();
        
        let graph_handle = state.key(state.models.keys());
        
        info!("load() - inserting graph: {:#?} with size {:#?}",
            graph_handle,
            model_bytes.len()
        );

        state.models.insert(graph_handle, model_bytes);

        let result_ok = LoadResult {
            has_error: false,
            runtime_error: None,
            guest_error: None,
            graph: Graph::from(graph_handle),
        };

        info!("load() - current number of models: {:#?}", state.models.len());

        Ok(result_ok)
    }

    /// init_execution_context
    async fn init_execution_context(
        &self,
        _ctx: &Context,
        arg: &Graph,
    ) -> RpcResult<IecResult> {
        let graph = GraphWrap::from(arg.graph);

        info!("init_execution_context: graph: {:#?}", graph);

        // TOD0: should not panic in case the lock is not available
        let mut state = self.state.write().unwrap();
        let mut model_bytes = match state.models.get(&graph) {
            Some(mb) => Cursor::new(mb),
            None => {
                error!("init_execution_context: cannot find model in state with graph {:#?}", graph);

                let result_with_error = IecResult {
                    has_error: true,
                    runtime_error: Some(RuntimeError::from(RuntimeErrorWrap::RuntimeError)),
                    guest_error: None, 
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
                    has_error: true,
                    runtime_error: None,
                    guest_error: Some(GuestError::from(GuestErrorWrap::ModelError)), 
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
            has_error: false,
            runtime_error: None,
            guest_error: None,
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
    async fn set_input(&self, _ctx: &Context, arg: &SetInputStruct) -> RpcResult<SetInputResult> {
        let mut state = self.state.write().unwrap();

        let gec_wrap = GECWrap::from(arg.context.gec);
        let index: u32 = match arg.index {
            Some(v) => v,
            None => 0,
        };
        let tensor = &arg.tensor;

        let execution = match state.executions.get_mut(&gec_wrap) {
            Some(s) => s,
            None => {
                error!("set_input() - cannot find session in state with context {:#?}", gec_wrap);

                let result_with_error = SetInputResult {
                    has_error: true,
                    runtime_error: Some(RuntimeError::from(RuntimeErrorWrap::ContextNotFound)),
                    guest_error: None
                };

                return Ok(result_with_error);
            }
        };

        let shape = tensor
        .dimensions
        .iter()
        .map(|d| *d as usize)
        .collect::<Vec<_>>();

        match execution.graph.set_input_fact(
        index as usize,
        InferenceFact::dt_shape(f32::datum_type(), shape.clone()),) {
            Ok(s) => s,
            Err(e) => {
                error!("set_input() - cannot set input fact {:#?}", e);

                let result_with_error = SetInputResult {
                    has_error: true,
                    runtime_error: Some(RuntimeError::from(RuntimeErrorWrap::RuntimeError)),
                    guest_error: None,
                };
                return Ok(result_with_error);
            }
        };
        
        let data = bytes_to_f32_vec(tensor.data.as_slice().to_vec())?;
        let input: TractTensor = match Array::from_shape_vec(shape, data){
            Ok(s) => s.into(),
            Err(e) => {
                error!("set_input() - corrupt tensor input {:#?}", e);

                let result_with_error = SetInputResult {
                    has_error: true,
                    runtime_error: None,
                    guest_error: Some(GuestError::from(GuestErrorWrap::CorruptInputTensor))
                };
                return Ok(result_with_error);
            }
        };

        match execution.input_tensors {
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

        let result_ok = SetInputResult {
            has_error: false,
            runtime_error: None,
            guest_error: None
        };

        Ok(result_ok)
    }
}

/// calculate n factorial
fn n_factorial(n: u32) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        _ => {
            let mut result = 1u64;
            // add 1 because rust ranges exclude upper bound
            for v in 2..(n + 1) {
                result *= v as u64;
            }
            result
        }
    }
}
