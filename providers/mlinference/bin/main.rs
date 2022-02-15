//! mlinference capability provider
//!

use std::{collections::HashMap, sync::Arc};
pub (crate) use wasmcloud_interface_mlinference::{
    Mlinference, MlinferenceReceiver, InferenceRequest, InferenceOutput, 
    MlError
};
use wasmcloud_provider_mlinference::{
    load_settings, get_default_inference_result, ModelZoo, ModelContext,
    TractEngine, InferenceEngine, Graph, GraphExecutionContext, BindleLoader
};
use tokio::sync::RwLock;
use bindle::{client::{Client, tokens::NoToken}};

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
    engine: TractEngine // could be arc of box of enum or
}

/// use default implementations of provider message handlers
impl ProviderDispatch for MlinferenceProvider {}

#[async_trait]
impl ProviderHandler for MlinferenceProvider {

    async fn put_link(&self, ld: &LinkDefinition) -> Result<bool, RpcError> 
    {
        let settings = load_settings(&ld.values)
            .map_err(|e| RpcError::ProviderInit(e.to_string()))?;

        let mut model_zoo: ModelZoo = ModelZoo::new();
        
        settings.models.zoo.iter().for_each(|(k,v)| {
            model_zoo
            .insert(k.to_string(), ModelContext { 
                bindle_url: v.to_string(), 
                ..Default::default()
            });
        });

        let bindle_client: Client<NoToken> = BindleLoader::provide("BINDLE_URL")
            .await
            .map_err(|error| RpcError::ProviderInit(format!("{}", error)))?;

        for (_, context) in model_zoo.iter_mut() 
        {           
            let downloads = BindleLoader::get_model_and_metadata(&bindle_client, &context.bindle_url)
                .await
                .map_err(|error| RpcError::ProviderInit(format!("{}", error)))?;

            let (metadata, model_data_bytes) = downloads;

            context.load_metadata(metadata)
            .map_err(|e| RpcError::InvalidParameter(format!("{:?}",e)))?;

            let graph: Graph = self.engine.load(&model_data_bytes, &context.graph_encoding, &context.execution_target)
                .await
                .map_err(|error| RpcError::ProviderInit(format!("{}", error)))?;

            context.graph = graph;

            let gec: GraphExecutionContext = self.engine.init_execution_context(context.graph)
                .await
                .map_err(|error| RpcError::ProviderInit(format!("{}", error)))?;

            context.graph_execution_context = gec;
        }

        log::debug!("==============> 'context' filled with {:?}", &model_zoo);

        let mut actor_lock = self.actors.write().await;
        actor_lock.insert(ld.actor_id.to_string(), model_zoo);

        log::info!("put_link() ==============>");

        Ok(true)
    }

    /// Handle notification that a link is dropped
    async fn delete_link(&self, actor_id: &str) 
    {
        let mut actor_lock = self.actors.write().await;

        let model_zoo: &ModelZoo = match actor_lock.get(actor_id) {
            Some(mz) => mz,
            None     => { return; }
        };

        for (_, context) in model_zoo.iter() 
        {
            self.engine.drop_model_state(&context.graph, &context.graph_execution_context).await;
        }

        actor_lock.remove(actor_id);
    }
}


/// Handle Mlinference methods
#[async_trait]
impl Mlinference for MlinferenceProvider {
    /// predict
    async fn predict(&self, ctx: &Context, arg: &InferenceRequest) -> RpcResult<InferenceOutput> 
    {  
        log::debug!("==============> predict()");
        
        let actor = match ctx.actor.as_ref() {
            Some(x) => x,
            None    => {
                let ir = get_default_inference_result(Some(MlError{err: 3}));
                return Ok(ir);
            }
        }.to_string();

        let model_name = &arg.model;

        let tensor_in = &arg.tensor;

        let index = arg.index;

        let ar = self.actors.read().await;

        let modelzoo: &ModelZoo = match ar.get(&actor) {
            Some(v) => v,
            None    => {
                let ir = get_default_inference_result(Some(MlError{err: 6}));
                return Ok(ir);
            }
        };

        let model_context: &ModelContext = match modelzoo.get(model_name) {
            Some(m) => m,
            None    => {
                let ir = get_default_inference_result(Some(MlError{err: 6}));
                return Ok(ir);
            }
        };

        match self.engine.set_input(model_context.graph_execution_context, index, tensor_in).await {
            Ok(_)    => {},
            Err(_)   => return Ok(get_default_inference_result(Some(MlError{err: 6})))
        }

        match self.engine.compute(model_context.graph_execution_context).await {
            Ok(_)    => {},
            Err(_)   => return Ok(get_default_inference_result(Some(MlError{err: 6})))
        }

        let result = match self.engine.get_output(model_context.graph_execution_context, index).await {
            Ok(r)    => r,
            Err(_)   => return Ok(get_default_inference_result(Some(MlError{err: 6})))
        };
        log::debug!("predict() ==============> ");
        Ok(result)
    }
}


// #[cfg(test)]
// mod tests {
    
//     // use std::{collections::HashMap};
//     // use wasmbus_rpc::core::{LinkDefinition, LinkSettings};
//     // use crate::MlinferenceProvider;
//     // use wasmbus_rpc::provider::ProviderHandler;

//     #[test]
//     fn it_works() {

//         // let x: LinkSettings = HashMap::from([
//         //     (String::from("flex"), String::from("enterprise.com/warpcore/1.2.0")),
//         //     (String::from("champion"), String::from("enterprise.com/warpcore/1.0.0")),
//         //     (String::from("challenger"), String::from("enterprise.com/warpcore/1.1.0")),
//         // ]);
    
//         // let link_definitions: LinkDefinition = LinkDefinition {
//         //     actor_id: "123".to_string(),
//         //     provider_id: "whatever".to_string(),
//         //     link_name: "whatever".to_string(),
//         //     contract_id: "unimportant".to_string(),
//         //     values: x
//         // };

//         // MlinferenceProvider.put_link(link_definitions);

//         //assert_eq!(2 + 2, 4);
//     }
// }