use serde::Deserialize;
use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_httpserver::{HttpRequest, HttpResponse, HttpServer, HttpServerReceiver};
use wasmcloud_interface_logging::{debug, error, warn};
use wasmcloud_interface_mlimagenet::{Imagenet, ImagenetSender};
use wasmcloud_interface_mlinference::{
    InferenceInput, InferenceOutput, MlInference, MlInferenceSender, Status, Tensor,
};
use wasmcloud_interface_mlpreprocessing::{
    ConversionRequest, MlPreprocessing, MlPreprocessingSender,
};

const IMAGENET_PREPROCESS_ACTOR: &str = "mlinference/imagenetpreprocessor";
const IMAGENET_PREPROCRGB8_ACTOR: &str = "mlinference/imagenetpreprocrgb8";
const IMAGENET_POSTPROCESS_ACTOR: &str = "mlinference/imagenetpostprocessor";

const MNIST_PREPROCESS_ACTOR: &str = "mlinference/mnistpreprocessor";
const MNIST_POSTPROCESS_ACTOR: &str = "mlinference/mnistpostprocessor";

#[derive(Debug, Default, Actor, HealthResponder)]
#[services(Actor, HttpServer)]
struct InferenceapiActor {}

/// Implementation of HttpServer trait methods
#[async_trait]
impl HttpServer for InferenceapiActor {
    async fn handle_request(
        &self,
        ctx: &Context,
        req: &HttpRequest,
    ) -> std::result::Result<HttpResponse, RpcError> {
        debug!("API request: {:?}", req);

        let path = &req.path[1..req.path.len()];
        let segments: Vec<&str> = path.trim_end_matches('/').split('/').collect();

        warn!("request {} {:?}", &req.method, &segments);

        match (req.method.as_ref(), segments.as_slice()) {
            ("POST", [model_name]) => {
                debug!("receiving POST(model) ..");

                // extract
                let tensor: Tensor = deser(&req.body).map_err(|error| {
                    error!("failed to deserialize the input tensor from POST body!");
                    RpcError::Deser(format!("{}", error))
                })?;

                // validate
                validate(model_name, &tensor).await?;

                // predict
                let prediction: InferenceOutput = predict(ctx, model_name, tensor).await?;

                if let Status::Error(error) = prediction.result {
                    Ok(HttpResponse::internal_server_error(format!(
                        "compute_output: {:?}",
                        error
                    )))
                } else {
                    HttpResponse::json(prediction, 200)
                }
            }

            ("PUT", [model_name]) => {
                debug!("receiving PUT(model) ..");

                // extract
                let tensor: Tensor = deser(&req.body).map_err(|error| {
                    error!("failed to deserialize the input tensor from PUT body!");
                    RpcError::Deser(format!("{}", error))
                })?;

                // validate
                validate(model_name, &tensor).await?;

                // predict
                let prediction: InferenceOutput = predict(ctx, model_name, tensor).await?;

                if let Status::Error(e) = prediction.result {
                    Ok(HttpResponse::internal_server_error(format!(
                        "compute_output: {:?}",
                        e
                    )))
                } else {
                    HttpResponse::json(prediction, 200)
                }
            }

            ("PUT", [model_name, "preprocess"]) => {
                debug!("receiving PUT(model, preprocess) ..");

                // preprocess
                let preprocessed = MlPreprocessingSender::to_actor(IMAGENET_PREPROCESS_ACTOR)
                    .convert(
                        ctx,
                        &ConversionRequest {
                            data: req.body.to_owned(),
                        },
                    )
                    .await?;

                // validate
                validate(model_name, &preprocessed.tensor).await?;

                // predict
                let prediction: InferenceOutput =
                    predict(ctx, model_name, preprocessed.tensor).await?;

                if let Status::Error(e) = prediction.result {
                    Ok(HttpResponse::internal_server_error(format!(
                        "compute_output: {:?}",
                        e
                    )))
                } else {
                    HttpResponse::json(prediction, 200)
                }
            }

            ("PUT", [model_name, "preprocess", "rgb8"]) => {
                debug!("receiving PUT(model, preprocess) ..");

                // preprocess
                let preprocessed = MlPreprocessingSender::to_actor(IMAGENET_PREPROCRGB8_ACTOR)
                    .convert(
                        ctx,
                        &ConversionRequest {
                            data: req.body.to_owned(),
                        },
                    )
                    .await?;

                // validate
                validate(model_name, &preprocessed.tensor).await?;

                // predict
                let prediction: InferenceOutput =
                    predict(ctx, model_name, preprocessed.tensor).await?;

                if let Status::Error(e) = prediction.result {
                    Ok(HttpResponse::internal_server_error(format!(
                        "compute_output: {:?}",
                        e
                    )))
                } else {
                    HttpResponse::json(prediction, 200)
                }
            }

            ("PUT", [model_name, "matches"]) => {
                debug!("receiving POST(model, classes) ..");

                // preprocess
                let preprocessed = MlPreprocessingSender::to_actor(IMAGENET_PREPROCESS_ACTOR)
                    .convert(
                        ctx,
                        &ConversionRequest {
                            data: req.body.to_owned(),
                        },
                    )
                    .await?;

                // validate
                validate(model_name, &preprocessed.tensor).await?;

                // predict
                let prediction: InferenceOutput =
                    predict(ctx, model_name, preprocessed.tensor).await?;

                // postprocess
                let postprocessed = ImagenetSender::to_actor(IMAGENET_POSTPROCESS_ACTOR)
                    .postprocess(ctx, &prediction)
                    .await?;

                if let Status::Error(e) = prediction.result {
                    Ok(HttpResponse::internal_server_error(format!(
                        "compute_output: {:?}",
                        e
                    )))
                } else {
                    HttpResponse::json(postprocessed, 200)
                }
            }

            ("PUT", [model_name, "matches", "rgb8"]) => {
                // preprocess
                let preprocessed = MlPreprocessingSender::to_actor(IMAGENET_PREPROCRGB8_ACTOR)
                    .convert(
                        ctx,
                        &ConversionRequest {
                            data: req.body.to_owned(),
                        },
                    )
                    .await?;

                // validate
                validate(model_name, &preprocessed.tensor).await?;

                // predict
                let prediction: InferenceOutput =
                    predict(ctx, model_name, preprocessed.tensor).await?;

                // postprocess
                let postprocessed = ImagenetSender::to_actor(IMAGENET_POSTPROCESS_ACTOR)
                    .postprocess(ctx, &prediction)
                    .await?;

                if let Status::Error(e) = prediction.result {
                    Ok(HttpResponse::internal_server_error(format!(
                        "compute_output: {:?}",
                        e
                    )))
                } else {
                    HttpResponse::json(postprocessed, 200)
                }
            }

            ("PUT", [model_name, "mnist", "matches"]) => {
                debug!("receiving PUT(model, classes) ..");

                // preprocess
                let preprocessed = MlPreprocessingSender::to_actor(MNIST_PREPROCESS_ACTOR)
                    .convert(
                        ctx,
                        &ConversionRequest {
                            data: req.body.to_owned(),
                        },
                    )
                    .await?;

                // validate
                validate(model_name, &preprocessed.tensor).await?;

                // predict
                let prediction: InferenceOutput =
                    predict(ctx, model_name, preprocessed.tensor).await?;

                // postprocess
                let postprocessed = ImagenetSender::to_actor(MNIST_POSTPROCESS_ACTOR)
                    .postprocess(ctx, &prediction)
                    .await?;

                if let Status::Error(e) = prediction.result {
                    Ok(HttpResponse::internal_server_error(format!(
                        "compute_output: {:?}",
                        e
                    )))
                } else {
                    HttpResponse::json(postprocessed, 200)
                }
            }

            (_, _) => {
                debug!("API request: {:?}", req);
                //Ok(HttpResponse::default())
                Ok(HttpResponse::internal_server_error("----N/A-----"))
            } //(_, _) => Ok(HttpResponse::not_found()),
        }
    }
}

async fn validate(model_name: &str, tensor: &Tensor) -> Result<(), RpcError> {
    if model_name.is_empty() {
        return Err(RpcError::InvalidParameter(
            "The name of a model MUST be provided!".to_string(),
        ));
    }

    if tensor.data.is_empty() {
        return Err(RpcError::InvalidParameter(
            "The input tensor MUST NOT be empty!".to_string(),
        ));
    }

    if tensor.dimensions.is_empty() {
        return Err(RpcError::InvalidParameter(
            "Tensor dimensions MUST be provided!".to_string(),
        ));
    }

    Ok(())
}

async fn predict(ctx: &Context, model_name: &str, tensor: Tensor) -> RpcResult<InferenceOutput> {
    debug!("Deserialized input tensor: {:?}", tensor);

    let input = InferenceInput {
        model: model_name.to_string(),
        index: 0,
        tensor,
    };

    let mls = MlInferenceSender::new();
    mls.predict(ctx, &input).await
}

fn deser<'de, T: Deserialize<'de>>(raw: &'de [u8]) -> RpcResult<T> {
    serde_json::from_slice(raw).map_err(|e| RpcError::Deser(format!("{}", e)))
}
