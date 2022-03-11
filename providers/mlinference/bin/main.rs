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
        log::error!("No 'BINDLE_URL' defined, verify your bindle url.");
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
        let this = self.clone();
        let ld = ld.clone();
        tokio::spawn(async move { this.put_link_sub(&ld).await });
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
    
impl MlinferenceProvider {
    async fn put_link_sub(&self, ld: &LinkDefinition) -> Result<bool, RpcError> 
    {
        log::debug!("put_link_sub() - link definition is '{:?}'", ld);
        
        let settings = load_settings(&ld.values)
            .map_err(|e| RpcError::ProviderInit(e.to_string()))?;

        log::debug!("put_link_sub() - just passed 'load_settings()'");

        let mut model_zoo: ModelZoo = ModelZoo::new();
        
        settings.models.zoo.iter().for_each(|(k,v)| {
            model_zoo
            .insert(k.to_string(), ModelContext { 
                bindle_url: v.to_string(), 
                ..Default::default()
            });
        });

        log::debug!("put_link_sub() - available content in modelzoo: '{:?}'", &model_zoo);

        let bindle_client: Client<NoToken> = BindleLoader::provide("BINDLE_URL")
            .await
            .map_err(|error| {
                log::error!("put_link_sub() no 'BINDLE_URL' found");
                RpcError::ProviderInit(format!("{}", error))
            })?;

        log::debug!("put_link_sub() - NOT done yet");

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

        {
            let mut actor_lock = self.actors.write().await;
            actor_lock.insert(ld.actor_id.to_string(), model_zoo);
        }

        log::debug!("put_link_sub() - DONE");

        Ok(true)
    }
}


/// Handle Mlinference methods
#[async_trait]
impl Mlinference for MlinferenceProvider {
    /// predict
    async fn predict(&self, ctx: &Context, arg: &InferenceRequest) -> RpcResult<InferenceOutput> 
    {      
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
                log::error!("predict() - returning early because no corresponding actor was found!");
                log::error!("predict() - actor supposed to be found '{}'", &actor);
                log::error!("predict() - available content in modelzoo: '{:?}'", &ar);
                return Ok(ir);
            }
        };

        let model_context: &ModelContext = match modelzoo.get(model_name) {
            Some(m) => m,
            None    => {
                let ir = get_default_inference_result(Some(MlError{err: 6}));
                log::error!("predict() - returning early because no corresponding model found!");
                return Ok(ir);
            }
        };

        let result = match self.engine.infer(model_context.graph_execution_context, index, tensor_in).await {
            Ok(r)    => r,
            Err(e)   => {
                log::error!("infer() - failed with '{}'", e);
                return Ok(get_default_inference_result(Some(MlError{err: 6})));
            }
        };

        // match self.engine.set_input(model_context.graph_execution_context, index, tensor_in).await {
        //     Ok(_)    => {},
        //     Err(e)   => {
        //         log::error!("predict() - inference engine failed in 'set_input()' with '{}'", e);
        //         return Ok(get_default_inference_result(Some(MlError{err: 6})));
        //     }
        // }

        // match self.engine.compute(model_context.graph_execution_context).await {
        //     Ok(_)    => {},
        //     Err(_)   => {
        //         log::error!("predict() - GraphExecutionContext not found");
        //         return Ok(get_default_inference_result(Some(MlError{err: 6})));
        //     }
        // }

        // let result = match self.engine.get_output(model_context.graph_execution_context, index).await {
        //     Ok(r)    => r,
        //     Err(_)   => {
        //         log::error!("predict() - could not gather results from 'get_output()'");
        //         return Ok(get_default_inference_result(Some(MlError{err: 6})));
        //     }
        // };

        log::debug!("predict() - PASSED, result is '{:?}'", &result);

        Ok(result)
    }
}