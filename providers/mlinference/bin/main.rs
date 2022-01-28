//! mlinference capability provider
//!

use std::{collections::HashMap, sync::Arc};
pub(crate) use wasmcloud_interface_mlinference::{
    Mlinference, MlinferenceReceiver, InferenceRequest, InferenceResult, Tensor
};
use wasmcloud_provider_mlinference::{
    load_settings, get_valid_status, ModelZoo, ModelContext, ModelName, ModelMetadata,
    get_first_member_of
};
use tokio::sync::RwLock;
use bindle::{client, invoice};
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

        for (model, context) in model_zoo.iter() 
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

            let model_data_blob = bindle_client
                .get_parcel(&context.bindle_url, &model_parcel.label.sha256)
                .await
                .map_err(|_| RpcError::ProviderInit(
                    format!("could not download model {} from path {}", model_parcel.label.name, &context.bindle_url)
                ))?;
            log::info!("successfully downloaded model {} of size {}", model_parcel.label.name, model_data_blob.len());

            let metadata_blob = bindle_client
                .get_parcel(&context.bindle_url, &metadata_parcel.label.sha256)
                .await
                .map_err(|_| RpcError::ProviderInit(
                    format!("could not download metadata {} from path {}", metadata_parcel.label.name, &context.bindle_url)
                ))?;
            log::info!("successfully downloaded metadata {} of size {}", metadata_parcel.label.name, metadata_blob.len());

            let metadata: ModelMetadata = ModelMetadata::from_json(&metadata_blob)
                .map_err(|error| RpcError::ProviderInit(
                    format!("{}", error)
                ))?;

           
        }
        

        // fetch the relevant url(s)
        let _inv = bindle_client.get_invoice("enterprise.com/warpcore/2.0.0").await;


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
            //drop_state(models);
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