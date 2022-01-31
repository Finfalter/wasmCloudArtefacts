//! mlinference capability provider
//!

use std::{collections::HashMap, sync::Arc};
pub (crate) use wasmcloud_interface_mlinference::{
    Mlinference, MlinferenceReceiver, InferenceRequest, InferenceOutput, 
    MlError
};
use wasmcloud_provider_mlinference::{
    load_settings, get_result_status, get_default_inference_result, ModelZoo, ModelContext, ModelMetadata,
    get_first_member_of, TractEngine, InferenceEngine, Graph, GraphExecutionContext
};
use tokio::sync::RwLock;
use bindle::{client};
//use log::{debug, info, error};  

use wasmbus_rpc::provider::prelude::*;


// main (via provider_main) initializes the threaded tokio executor,
// listens to lattice rpcs, handles actor links,
// and returns only when it receives a shutdown message
//
fn main() -> Result<(), Box<dyn std::error::Error>> {
    provider_main(MlinferenceProvider::default())?;

    if std::env::var("BINDLE_URL").is_err() {
        return Err("No 'BINDLE_URL' defined, verify your bindle url.".into())
    }

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
    engine: TractEngine
}

/// use default implementations of provider message handlers
impl ProviderDispatch for MlinferenceProvider {}

#[async_trait]
impl ProviderHandler for MlinferenceProvider {

    async fn put_link(&self, ld: &LinkDefinition) -> Result<bool, RpcError> {
        let settings = load_settings(&ld.values).map_err(|e| RpcError::ProviderInit(e.to_string()))?;

        //let model_zoo: ModelZoo = settings.models.zoo;
        let mut model_zoo: ModelZoo = ModelZoo::new();
        
        settings.models.zoo.iter().for_each(|(k,v)| {
            model_zoo
            .insert(k.to_string(), ModelContext{ 
                bindle_url: v.to_string(), 
                ..Default::default() 
            });
        });
        
        // init the connection to bindle
        let url = std::env::var("BINDLE_URL")
            .map_err(|_| RpcError::InvalidParameter("No 'BINDLE_URL' defined, verify your bindle url.".into()))?;
        
        let bindle_client = client::Client::new(&url, client::tokens::NoToken)
            .map_err(|_| RpcError::InvalidParameter("'BINDLE_URL' not valid, verify your bindle url.".into()))?;

        for (_, context) in model_zoo.iter_mut() 
        {
            let invoice = bindle_client.get_invoice(&context.bindle_url).await
                .map_err(|_| RpcError::InvalidParameter(
                    format!("invoice '{}' could not be fetched", &context.bindle_url)
                ))?;

            let parcels = invoice.parcel.ok_or(
                RpcError::InvalidParameter(
                    format!("invoice '{}' could not be fetched", &context.bindle_url)
                ))?;

            let model_parcel = get_first_member_of(&parcels, "model")
                .map_err(|_| RpcError::InvalidParameter(
                    format!("The invoice must have >0 parcels being member of group 'model'")
                ))?;

            let metadata_parcel = get_first_member_of(&parcels, "metadata")
                .map_err(|_| RpcError::InvalidParameter(
                    format!("The invoice must have >0 parcels being member of group 'metadata'")
                ))?;

            let model_data_blob: Vec<u8> = bindle_client
                .get_parcel(&context.bindle_url, &model_parcel.label.sha256)
                .await
                .map_err(|_| RpcError::ProviderInit(
                    format!("could not download model {} from path {}", 
                    model_parcel.label.name, &context.bindle_url)
                ))?;
            log::info!("successfully downloaded model {} of size {}", 
                model_parcel.label.name, model_data_blob.len());

            let metadata_blob: Vec<u8> = bindle_client
                .get_parcel(&context.bindle_url, &metadata_parcel.label.sha256)
                .await
                .map_err(|_| RpcError::ProviderInit(
                    format!("could not download metadata {} from path {}", 
                        metadata_parcel.label.name, &context.bindle_url)
                ))?;
            log::info!("successfully downloaded metadata {} of size {}", 
                metadata_parcel.label.name, metadata_blob.len());

            // storing metadata makes sense when model data is done
            let metadata: ModelMetadata = ModelMetadata::from_json(&metadata_blob)
                .map_err(|error| RpcError::ProviderInit(
                    format!("{}", error)
                ))?;

            context.clone().load_metadata(metadata)
                .map_err(|e| RpcError::InvalidParameter(format!("{:?}",e)))?;

            let graph: Graph = self.engine.load(&model_data_blob, &context.graph_encoding, &context.execution_target)
                .await
                .map_err(|error| RpcError::ProviderInit(
                    format!("{}", error)
                ))?;

            context.graph = graph;

            let gec: GraphExecutionContext = self.engine.init_execution_context(context.graph)
                .await
                .map_err(|error| RpcError::ProviderInit(
                    format!("{}", error)
                ))?;

            context.session = gec;
        }

        let mut actor_lock = self.actors.write().await;
        actor_lock.insert(ld.actor_id.to_string(), model_zoo);

        Ok(true)
    }

    /// Handle notification that a link is dropped
    /// remove the corresponding actor from the list
    /// TODO__CB__ cleanup underlying resources 
    async fn delete_link(&self, actor_id: &str) {
        let mut aw = self.actors.write().await;
        if let Some(models) = aw.remove(actor_id) {
            // remove all state for this actor-link's pool
            //drop_state(models);
        }
    }
}


/// Handle Mlinference methods
#[async_trait]
impl Mlinference for MlinferenceProvider {
    /// predict
    async fn predict(&self, ctx: &Context, arg: &InferenceRequest) -> RpcResult<InferenceOutput> {
      
        let actor = match ctx.actor.as_ref() {
            Some(x) => x,
            None    => {
                let ir = get_default_inference_result(Some(MlError{model_error: 0}));
                return Ok(ir);
            }
        }.to_string();

        let model = arg.model.clone();

        let tensor_in = arg.tensor.clone();

        let index = arg.index;

        let actor_access = self.actors.read().await;

        let modelzoo: ModelZoo = match actor_access.get(&actor) {
            Some(v) => v,
            None    => {
                let ir = get_default_inference_result(Some(MlError{model_error: 1}));
                return Ok(ir);
            }
        };

        // TODO__CB__: refactor MlError 
        //self.engine.set_input(modelzoo.);

        

        Ok(get_default_inference_result(None))
    }
}