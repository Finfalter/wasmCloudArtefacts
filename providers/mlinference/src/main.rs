//! mlinference capability provider
//!

use std::{collections::HashMap, convert::Infallible, sync::Arc};
pub(crate) use wasmcloud_interface_mlinference::{
    Mlinference, MlinferenceReceiver, InferenceRequest, InferenceResult, Tensor
};
use mlinference::{load_settings, get_valid_status, drop_state, ModelZoo};
use tokio::sync::RwLock;
//use log::{debug, info, error};  

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
    /// map to store the assignments between the respective model 
    /// and corresponding bindle path for each linked actor
    actors: Arc<RwLock<HashMap<String, ModelZoo>>>,
}

/// use default implementations of provider message handlers
impl ProviderDispatch for MlinferenceProvider {}

#[async_trait]
impl ProviderHandler for MlinferenceProvider {

    async fn put_link(&self, ld: &LinkDefinition) -> Result<bool, RpcError> {
        let settings = load_settings(&ld.values).map_err(|e| RpcError::ProviderInit(e.to_string()))?;

        let model_zoo: ModelZoo = settings.models.zoo;

        let mut update_map = self.actors.write().await;
        update_map.insert(ld.actor_id.to_string(), model_zoo);

        Ok(true)
    }

    /// Handle notification that a link is dropped
    /// remove the corresponding actor from the list
    /// TODO__CB__ cleanup underlying resources 
    async fn delete_link(&self, actor_id: &str) {
        let mut aw = self.actors.write().await;
        if let Some(models) = aw.remove(actor_id) {
            // remove all state for this actor-link's pool
            drop_state(models);
        }
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