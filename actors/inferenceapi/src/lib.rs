use serde::Deserialize;
use serde_json;
use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_httpserver::{HttpRequest, HttpResponse, HttpServer, HttpServerReceiver};
use wasmcloud_interface_logging::{debug};
use wasmcloud_interface_mlinference::{
    InferenceInput, InferenceOutput, MlInference, MlInferenceSender, Status, Tensor, ValueType
};
use wasmcloud_interface_mlpreprocessing::{MlPreprocessingSender, MlPreprocessing, ConversionRequest};

const PREPROCESS_ACTOR: &str = "mlinference/imagepreprocessor";

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

        debug!("Segments: {:?}", segments);

        match (req.method.as_ref(), segments.as_slice()) {

            ("POST", [model_name]) => {
                debug!("receiving POST(model) ..");
                
                // extract
                let tensor: Tensor = deser(&req.body).map_err(|error| {
                    log::error!("failed to deserialize the input tensor from POST body!");
                    RpcError::Deser(format!("{}", error))
                })?;

                // validate
                validate(model_name, &tensor).await.map_err(|error| error).ok();
                    
                let prediction:InferenceOutput = predict(ctx, model_name, tensor).await?;
                //let prediction = get_prediction(ctx, model_name, "0", tensor).await?;

                if let Status::Error(error) = prediction.result {
                    Ok(HttpResponse::internal_server_error(format!(
                        "compute_output: {:?}", error
                    )))
                } else {
                    HttpResponse::json(prediction, 200)
                }
            }

            ("POST", [model_name, "preprocess"]) => {
                debug!("receiving POST(model, preprocess) ..");
                
                let preprocessed = MlPreprocessingSender::to_actor(PREPROCESS_ACTOR)
                    .convert(ctx, &ConversionRequest { data: req.body.to_owned() })
                    .await?;                

                let tensor = preprocessed.tensor;

                // // validate
                // validate(model_name, tensor).await
                // .map_err(|error| Ok(HttpResponse::internal_server_error(
                //     format!("Invalid input arguments: {}", error))
                // ));

                //let prediction = get_prediction(ctx, model_name, "0", tensor).await?;
                let prediction:InferenceOutput = predict(ctx, model_name, tensor).await?;

                if let Status::Error(e) = prediction.result {
                    Ok(HttpResponse::internal_server_error(format!(
                        "compute_output: {:?}",
                        e
                    )))
                } else {
                    HttpResponse::json(prediction, 200)
                }
            }

            ("POST", [model_name, "classes"]) => {
                debug!("receiving POST(model, classes) ..");
                
                let preprocessed = MlPreprocessingSender::to_actor(PREPROCESS_ACTOR)
                    .convert(ctx, &ConversionRequest { data: req.body.to_owned() })
                    .await?;
                
                let tensor = preprocessed.tensor;

                // // validate
                // validate(model_name, tensor).await
                // .map_err(|error| Ok(HttpResponse::internal_server_error(
                //     format!("Invalid input arguments: {}", error))
                // ));

                let tensor = Tensor {
                    value_types: vec![ ValueType::ValueF32 ],
                    dimensions: tensor.dimensions,
                    data: tensor.data,
                    flags: tensor.flags,
                };

                //let prediction = get_prediction(ctx, model_name, "0", tensor).await?;
                let prediction:InferenceOutput = predict(ctx, model_name, tensor).await?;

                // postprocess

                if let Status::Error(e) = prediction.result {
                    Ok(HttpResponse::internal_server_error(format!(
                        "compute_output: {:?}",
                        e
                    )))
                } else {
                    HttpResponse::json(prediction, 200)
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

async fn validate(
    model_name: &str,
    tensor: &Tensor,
) -> Result<(), HttpResponse> {

    if model_name.is_empty()  {
        return Err(HttpResponse::internal_server_error("The name of a model MUST be provided!".to_string()));
    }

    if tensor.data.is_empty()  {
        return Err(HttpResponse::internal_server_error("The input tensor MUST NOT be empty!".to_string()));
    }

    if tensor.dimensions.is_empty()  {
        return Err(HttpResponse::internal_server_error("Tensor dimensions MUST be provided!".to_string()));
    }
    Ok(())
}

async fn predict(
    ctx: &Context,
    model_name: &str,
    tensor: Tensor,
) -> RpcResult<InferenceOutput> {

    debug!("Deserialized input tensor: {:?}", tensor);

    let input = InferenceInput {
        model: model_name.to_string(),
        index: 0,
        tensor: tensor,
    };

    let mls = MlInferenceSender::new();
    mls.predict(ctx, &input).await
}

// async fn get_prediction(
//     ctx: &Context,
//     model_name: &str,
//     index: &str,
//     tensor: Tensor,
// ) -> RpcResult<HttpResponse> {
//     debug!("Deserialized input tensor: {:?}", tensor);

//     if model_name.is_empty() || tensor.data.is_empty() || tensor.dimensions.is_empty() {
//         return Ok(HttpResponse::internal_server_error(
//             "Invalid input arguments",
//         ));
//     }

//     let input = InferenceInput {
//         model: model_name.to_string(),
//         index: index.parse().unwrap_or(0),
//         tensor: tensor,
//     };

//     let mls = MlInferenceSender::new();
//     let prediction = mls.predict(ctx, &input).await?;

//     if let Status::Error(e) = prediction.result {
//         Ok(HttpResponse::internal_server_error(format!(
//             "compute_output: {:?}",
//             e
//         )))
//     } else {
//         HttpResponse::json(prediction, 200)
//     }
// }

fn deser<'de, T: Deserialize<'de>>(raw: &'de [u8]) -> RpcResult<T> {
    serde_json::from_slice(raw).map_err(|e| RpcError::Deser(format!("{}", e)))
}
