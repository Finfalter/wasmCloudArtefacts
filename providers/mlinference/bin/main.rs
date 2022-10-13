//! mlinference capability provider
//!
use bindle::client::{tokens::NoToken, Client};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use wasmbus_rpc::provider::prelude::*;
pub(crate) use wasmcloud_interface_mlinference::{
    InferenceInput, InferenceOutput, MlError, MlInference, MlInferenceReceiver,
};

#[cfg(any(feature = "tflite", feature = "edgetpu"))]
use wasmcloud_provider_mlinference::TfLiteEngine;

use wasmcloud_provider_mlinference::{
    get_default_inference_result, load_settings, BindleLoader, Engine, Graph, GraphEncoding,
    GraphExecutionContext, InferenceFramework, ModelContext, ModelZoo, TractEngine,
};


/// main (via provider_main) initializes the threaded tokio executor,
/// listens to lattice rpcs, handles actor links,
/// and returns only when it receives a shutdown message
///
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_log::LogTracer::init()?;

    provider_main(
        MlInferenceProvider::default(),
        Some("mlinference".to_string()),
    )?;

    if std::env::var("BINDLE_URL").is_err() {
        log::error!("No 'BINDLE_URL' defined, verify your bindle url.");
        return Err("No 'BINDLE_URL' defined, verify your bindle url.".into());
    }

    eprintln!("mlinference provider exiting");
    Ok(())
}

/// mlinference capability provider implementation
#[derive(Default, Clone, Provider)]
#[services(MlInference)]
struct MlInferenceProvider {
    /// map to store the assignments between the respective model
    /// and corresponding bindle path for each linked actor
    /// TODO:
    ///   - instead of delaying putLink for model loading and initialization,
    ///     add a Ready flag (AtomicBool) that is set after model is loaded and initialized.
    ///   - initialize actor link as soon as we receive the putlink command
    ///   - if health check or rpc is received when not ready, return not-ready error
    actors: Arc<RwLock<HashMap<String, ModelZoo>>>,

    /// There are the following relevant types:
    ///     - InferenceFramework
    ///     - Engine
    ///     - InferenceEngine
    ///     - GraphEncoding
    ///
    /// InferenceFramework corresponds to an integrated crate.
    /// An InferenceFramework may support a multitude of Engine
    /// An InferenceFramework may support a multitude of GraphEncoding.
    /// Engine is a wrapper of InferenceEngine
    /// InferenceEngine defines common behavior
    /// GraphEncoding defines a model's encoding.
    engines: Arc<RwLock<HashMap<InferenceFramework, Engine>>>,
}

/// use default implementations of provider message handlers
impl ProviderDispatch for MlInferenceProvider {}

#[async_trait]
impl ProviderHandler for MlInferenceProvider {
    async fn put_link(&self, ld: &LinkDefinition) -> Result<bool, RpcError> {
        let mut this = self.clone();
        let ld = ld.clone();
        tokio::spawn(async move { this.put_link_sub(&ld).await });
        Ok(true)
    }

    /// Handle notification that a link is dropped
    async fn delete_link(&self, actor_id: &str) {
        let mut actor_lock = self.actors.write().await;

        let model_zoo: &ModelZoo = match actor_lock.get(actor_id) {
            Some(mz) => mz,
            None => {
                return;
            }
        };

        for (_, context) in model_zoo.iter() {
            let engine = match self.get_engine(&context).await {
                Ok(v) => v,
                Err(_) => {
                    return;
                }
            };

            engine
                .drop_model_state(&context.graph, &context.graph_execution_context)
                .await;
        }

        actor_lock.remove(actor_id);
    }
}

impl MlInferenceProvider {
    async fn put_link_sub(&mut self, ld: &LinkDefinition) -> Result<bool, RpcError> {
        log::debug!("put_link_sub() - link definition is '{:?}'", ld);

        let settings =
            load_settings(&ld.values).map_err(|e| RpcError::ProviderInit(e.to_string()))?;

        log::debug!("put_link_sub() - just passed 'load_settings()'");

        let mut model_zoo: ModelZoo = ModelZoo::new();

        settings.models.zoo.iter().for_each(|(k, v)| {
            model_zoo.insert(
                k.to_string(),
                ModelContext {
                    bindle_url: v.to_string(),
                    ..ModelContext::default()
                },
            );
        });

        log::debug!(
            "put_link_sub() - available content in modelzoo: '{:?}'",
            &model_zoo
        );

        let bindle_client: Client<NoToken> =
            BindleLoader::provide("BINDLE_URL").await.map_err(|error| {
                log::error!("put_link_sub() no 'BINDLE_URL' found");
                RpcError::ProviderInit(format!("{}", error))
            })?;

        log::debug!("put_link_sub() - NOT done yet");

        for (_, context) in model_zoo.iter_mut() {
            let downloads =
                BindleLoader::get_model_and_metadata(&bindle_client, &context.bindle_url)
                    .await
                    .map_err(|error| {
                        log::error!("get_model_and_metadata() failed!");
                        RpcError::ProviderInit(format!("{}", error))
                    })?;

            let (metadata, model_data_bytes) = downloads;

            context.load_metadata(metadata).map_err(|error| {
                log::error!("load_metadata() failed!");
                RpcError::InvalidParameter(format!("{:?}", error))
            })?;

            // each link definition may address a different target
            // such that it may be necessary to support multiple engines.
            let engine = self.get_or_else_set_engine(&context).await?;

            let graph: Graph = engine
                .load(&model_data_bytes)
                .await
                .map_err(|error| RpcError::ProviderInit(format!("{}", error)))?;

            context.graph = graph;

            let gec: GraphExecutionContext = engine
                .init_execution_context(
                    context.graph,
                    &context.execution_target,
                    &context.graph_encoding,
                )
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

/// Handle MlInference methods
#[async_trait]
impl MlInference for MlInferenceProvider {
    /// predict
    async fn predict(&self, ctx: &Context, arg: &InferenceInput) -> RpcResult<InferenceOutput> {
        let actor = match ctx.actor.as_ref() {
            Some(x) => x,
            None => {
                let ir = get_default_inference_result(Some(MlError::RuntimeError("".into())));
                return Ok(ir);
            }
        }
        .to_string();

        let model_name = &arg.model;
        let index = arg.index;

        let ar = self.actors.read().await;
        let modelzoo: &ModelZoo = match ar.get(&actor) {
            Some(v) => v,
            None => {
                let ir =
                    get_default_inference_result(Some(MlError::ContextNotFoundError("".into())));
                log::error!("predict() - actor {} not found, modelzoo={:?}", &actor, &ar);
                return Ok(ir);
            }
        };

        let model_context: ModelContext = match modelzoo.get(model_name) {
            Some(m) => m.clone(),
            None => {
                let ir = get_default_inference_result(Some(MlError::ContextNotFoundError(
                    model_name.clone(),
                )));
                log::error!("predict() - returning early because no corresponding model found!");
                return Ok(ir);
            }
        };

        let engine = self.get_engine(&model_context).await?;

        // it could be an expensive operation to clone the tensor,
        // but we hope (unconfirmed) the compiler will recognize that
        // the caller (dispatch fn) doesn't need it anymore and optimize out the clone.
        // TODO: confirm that this is true, or else find a way to make arg owned or Cow<'a>
        let tensor_in = arg.tensor.to_owned();
        let result = tokio::task::spawn_blocking(move || async move {
            if let Err(e) = engine
                .set_input(model_context.graph_execution_context, index, &tensor_in)
                .await
            {
                log::error!(
                    "predict() - inference engine failed in 'set_input()' with '{}'",
                    e
                );
                return get_default_inference_result(Some(MlError::ContextNotFoundError(
                    e.to_string(),
                )));
            }

            if let Err(e) = engine.compute(model_context.graph_execution_context).await {
                log::error!("predict() - GraphExecutionContext not found: {}", e);
                return get_default_inference_result(Some(MlError::ContextNotFoundError(
                    e.to_string(),
                )));
            }
            match engine
                .get_output(model_context.graph_execution_context, index)
                .await
            {
                Ok(result) => result,
                Err(e) => {
                    log::error!("predict() - could not gather results from 'get_output()'");
                    get_default_inference_result(Some(MlError::ContextNotFoundError(e.to_string())))
                }
            }
        })
        .await
        .map_err(|e| RpcError::Other(format!("internal join error: {}", e)))?
        .await;

        log::debug!("predict() - PASSED, result is '{:?}'", &result);
        Ok(result)
    }
}

impl MlInferenceProvider {
    /// Each link definition may address a different target
    /// such that it may be necessary to support multiple engines.
    async fn get_or_else_set_engine(&mut self, context: &ModelContext) -> Result<Engine, RpcError> {
        let mut engines_lock = self.engines.write().await;

        match context.graph_encoding {
            GraphEncoding::Onnx | GraphEncoding::Tensorflow => {
                match engines_lock.get(&InferenceFramework::Tract) {
                    Some(e) => {
                        log::debug!("get_or_else_set_engine() - previously created Onnx/Tensorflow engine selected for '{}'.", context.bindle_url);
                        Ok(e.clone())
                    }
                    None => {
                        let feedback = engines_lock.insert(
                            InferenceFramework::Tract,
                            Arc::new(Box::new(TractEngine::default())),
                        );
                        log::debug!("get_or_else_set_engine() - Onnx/Tensorflow engine selected and created for '{}'.", context.bindle_url);

                        if feedback.is_none() {
                            log::debug!(
                                "get_or_else_set_engine() - Onnx/Tensorflow was definitely created"
                            );
                        } else {
                            log::debug!(
                                "get_or_else_set_engine() - Onnx/Tensorflow was NOT created!"
                            );
                        }

                        log::debug!(
                            "get_or_else_set_engine() - content of Engines: {:?}: with length {:?}",
                            &engines_lock.keys(),
                            &engines_lock.keys().len()
                        );

                        //assert_eq!(engines_lock.is_empty(), false);

                        Ok(engines_lock
                            .get(&InferenceFramework::Tract)
                            .unwrap()
                            .clone())
                    }
                }
            }

            #[cfg(any(feature = "tflite", feature = "edgetpu"))]
            GraphEncoding::TfLite => {
                match engines_lock.get(&InferenceFramework::TfLite) {
                    Some(e) => {
                        log::debug!("get_or_else_set_engine() - previously created TfLite engine selected for '{}'.", context.bindle_url);
                        Ok(e.clone())
                    }
                    None => {
                        let feedback = engines_lock.insert(
                            InferenceFramework::TfLite,
                            Arc::new(Box::new(TfLiteEngine::default())),
                        );
                        log::debug!("get_or_else_set_engine() - TfLite engine selected and created for '{}'.", context.bindle_url);

                        if feedback.is_none() {
                            log::debug!(
                                "get_or_else_set_engine() - TfLite engine was definitely created"
                            );
                        } else {
                            log::debug!("get_or_else_set_engine() - TfLite was NOT created!");
                        }

                        log::debug!(
                            "get_or_else_set_engine() - content of Engines: {:?}: with length {:?}",
                            &engines_lock.keys(),
                            &engines_lock.keys().len()
                        );

                        Ok(engines_lock
                            .get(&InferenceFramework::TfLite)
                            .unwrap()
                            .clone())
                    }
                }
            }
            _ => {
                log::error!(
                    "unsupported graph encoding detected '{:?}'",
                    &context.graph_encoding
                );
                Err(RpcError::NotImplemented)
            }
        }
    }

    async fn get_engine(&self, context: &ModelContext) -> Result<Engine, RpcError> {
        let engines_lock = self.engines.write().await;

        log::debug!("get_engine() - context: {:?}", &context);

        match context.graph_encoding {
            GraphEncoding::Onnx | GraphEncoding::Tensorflow => {
                match engines_lock.get(&InferenceFramework::Tract) {
                    Some(e) => Ok(e.clone()),
                    None => Err(RpcError::InvalidParameter(
                        "get_engine() - No engine found for graph encoding 'Onnx' or 'Tensorflow'"
                            .to_string(),
                    )),
                }
            }
            GraphEncoding::TfLite => match engines_lock.get(&InferenceFramework::TfLite) {
                Some(e) => Ok(e.clone()),
                None => Err(RpcError::InvalidParameter(
                    "get_engine() - No engine found for graph encoding 'TfLite'".to_string(),
                )),
            },
            _ => {
                log::error!(
                    "get_engine() - unsupported graph encoding detected '{:?}'",
                    &context.graph_encoding
                );
                Err(RpcError::NotImplemented)
            }
        }
    }
}
