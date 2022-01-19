//! mlinference capability provider
//!

//mod lib;
//mod settings;


use std::{collections::HashMap, convert::Infallible, sync::Arc};
pub(crate) use wasmcloud_interface_mlinference::{
    Mlinference, MlinferenceReceiver, InferenceRequest, InferenceResult, Tensor
};
use mlinference::{load_settings, get_valid_status, ModelName, BindlePath};
use tokio::sync::RwLock;
use log::{debug, info, error};  

use wasmbus_rpc::provider::prelude::*;


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

#[async_trait]
impl ProviderHandler for MlinferenceProvider {
    /// Provider should perform any operations needed for a new link,
    /// including setting up per-actor resources, and checking authorization.
    /// If the link is allowed, return true, otherwise return false to deny the link.
    /// This message is idempotent - provider must be able to handle
    /// duplicates
    #[allow(unused_variables)]
    async fn put_link(&self, ld: &LinkDefinition) -> Result<bool, RpcError> {
        let settings =
            load_settings(&ld.values).map_err(|e| RpcError::ProviderInit(e.to_string()))?;

        Ok(true)
    }

    /// Notify the provider that the link is dropped
    #[allow(unused_variables)]
    async fn delete_link(&self, actor_id: &str) {}

    /// Handle system shutdown message
    async fn shutdown(&self) -> Result<(), Infallible> {
        Ok(())
    }
}


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