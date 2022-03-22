use serde::Deserialize;
use serde_json;
use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_httpserver::{HttpRequest, HttpResponse, HttpServer, HttpServerReceiver};
use wasmcloud_interface_logging::debug;
use wasmcloud_interface_mlinference::{InferenceRequest, Mlinference, MlinferenceSender, Tensor};

//const INFERENCE_ACTOR: &str = "mlinference/predict";

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
            ("POST", ["model", model_name, "index", index]) => {
                let tensor: Tensor = deser(&req.body)
                .map_err(|error| {
                    log::error!("failed to deserialize the input tensor from POST body!");
                    RpcError::Deser(format!("{}", error))
                })?;

                log::debug!("receiving POST(model_name, index) ..");

                get_prediction(ctx, model_name, index, tensor).await
            }
            (_, _) => {
                debug!("API request: {:?}", req);
                //Ok(HttpResponse::default())
                Ok(HttpResponse::internal_server_error("----N/A-----"))
            } //(_, _) => Ok(HttpResponse::not_found()),
        }
    }
}

async fn get_prediction(
    ctx: &Context,
    model_name: &str,
    index: &str,
    tensor: Tensor,
) -> RpcResult<HttpResponse> {
    debug!("Deserialized input tensor: {:?}", tensor);

    if model_name.is_empty() || tensor.data.is_empty() || tensor.dimensions.is_empty() {
        return Ok(HttpResponse::internal_server_error(
            "Invalid input arguments",
        ));
    }

    let co_re = InferenceRequest {
        model: model_name.to_string(),
        index: index.parse().unwrap_or(0),
        tensor: tensor,
    };

    let mls = MlinferenceSender::new();
    let compute_output = mls.predict(ctx, &co_re).await?;

    if !compute_output.result.has_error {
        HttpResponse::json(compute_output, 200)
    } else {
        Ok(HttpResponse::internal_server_error(
            format!("compute_output: {:?}", compute_output), 
        ))
    }
}

fn deser<'de, T: Deserialize<'de>>(raw: &'de [u8]) -> RpcResult<T> {
    serde_json::from_slice(raw).map_err(|e| RpcError::Deser(format!("{}", e)))
}
