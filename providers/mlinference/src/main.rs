//! mlinference capability provider
//!
mod lib;

use lib::{ BindlePath, ModelName, get_valid_status};
use tokio::sync::RwLock;
use log::{debug, info, error};  
use wasmbus_rpc::provider::prelude::*;
use wasmcloud_interface_mlinference::{
    Mlinference, MlinferenceReceiver, InferenceRequest, InferenceResult,
    Tensor
};

use std::{collections::HashMap, sync::Arc};

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
    // map to store http server (and its link parameters) for each linked actor
    actors: Arc<RwLock<HashMap<ModelName, BindlePath>>>,
}

/// use default implementations of provider message handlers
impl ProviderDispatch for MlinferenceProvider {}
impl ProviderHandler for MlinferenceProvider {}

/// Handle Mlinference methods
#[async_trait]
impl Mlinference for MlinferenceProvider {
    /// compute
    async fn compute(&self, _ctx: &Context, _arg: &InferenceRequest) -> RpcResult<InferenceResult> {
        let ir = InferenceResult {
            result: get_valid_status(),
            tensor: Tensor {
                data: vec![],
                dimensions: vec![]
            }
        };

        Ok(ir)
    }
}