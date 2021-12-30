//! mlinference capability provider
//!

//use tract::{State};
mod tract;

use tract::{GraphWrap, GraphEncoding, GuestErrorWrap};

use std::{
    collections::{btree_map::Keys, BTreeMap},
    sync::{Arc, RwLock},
};
use log::{debug, info, error};
use wasmbus_rpc::provider::prelude::*;
use wasmcloud_interface_mlinference::{Mlinference, MlinferenceReceiver, LoadInput, Graph, LoadResult, GuestError};

// main (via provider_main) initializes the threaded tokio executor,
// listens to lattice rpcs, handles actor links,
// and returns only when it receives a shutdown message
//
fn main() -> Result<(), Box<dyn std::error::Error>> {
    provider_main(MlinferenceProvider::default())?;

    eprintln!("mlinference provider exiting");
    Ok(())
}

#[derive(Default)]
pub struct State {
    //pub executions: BTreeMap<GraphExecutionContext, TractSession>,
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
