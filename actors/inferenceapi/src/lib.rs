use serde::{Deserialize};
use serde_json;
use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_logging::debug;
use wasmcloud_interface_httpserver::{HttpRequest, HttpResponse, HttpServer, HttpServerReceiver};
use wasmcloud_interface_mlinference::{Mlinference, MlinferenceSender, InferenceRequest, Tensor};

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

        match (req.method.as_ref(), segments.as_slice()) 
        {
            ("GET", ["vets"]) => Ok(HttpResponse::default()),
            //("POST", ["model", _model_name, "index", index]) => { 
            // ("POST", ["model", model_name, "index", index]) => { 
            //     Ok(HttpResponse {
            //         body: format!("Hello model {} with index {}", model_name, index).as_bytes().to_vec(),
            //         ..Default::default()
            //     })
            // },
            // headers, e.g. the header of a GET request, have size limitations, tpyically ~10k
            ("POST", ["model", model_name, "index", index]) => { 
                //let _x: Tensor = serde_json::from_slice(&req.body).unwrap();
                //get_prediction(ctx, model_name, index, &req.body).await;
                //debug!("WHAT WE GOT {:?}", deser(&req.body)?);
                //debug!("WHAT WE GOT {:?}", &req.body);
                let tensor: Tensor = deser(&req.body).unwrap();

                debug!("TENSOR: {:?}", &tensor);

                get_prediction(ctx, model_name, index, tensor).await

                // Ok(HttpResponse {
                //     //body: format!("Hello model {} with index {} and {:?}", model_name, index, &req.body).as_bytes().to_vec(),
                //     body: format!("Hello model {} with index {}", model_name, index).as_bytes().to_vec(),
                //     ..Default::default()
                // })
            },
            (_, _) => {
                debug!("API request: {:?}", req);
                //Ok(HttpResponse::default())
                Ok(HttpResponse::internal_server_error("----N/A-----",))
            }
            //(_, _) => Ok(HttpResponse::not_found()),
        }
    }
}

async fn get_prediction(
    ctx: &Context,
    model_name: &str,
    index: &str,
    tensor: Tensor,
) -> RpcResult<HttpResponse> 
{
    debug!("TENSOR: {:?}", tensor);

    if model_name.is_empty() || tensor.data.is_empty() || tensor.dimensions.is_empty() {
        return Ok(HttpResponse::internal_server_error("Invalid input arguments",));
    }
    
    let co_re = InferenceRequest {
        model: model_name.to_string(),
        index: index.parse().unwrap_or(0),
        tensor: tensor
    };

    //let compute_output: InferenceOutput = MlinferenceSender::to_actor(INFERENCE_ACTOR).predict(ctx, &co_re).await?;
    let mls = MlinferenceSender::new();
    let compute_output = mls.predict(ctx, &co_re).await?;

    if ! compute_output.result.has_error {
        HttpResponse::json(compute_output, 200)
    } else {
        Ok(HttpResponse::internal_server_error(
            format!("compute_output: {:?}", compute_output)
            //"Failed to compute",
        ))
    }
}

fn deser<'de, T: Deserialize<'de>>(raw: &'de [u8]) -> RpcResult<T> {
    serde_json::from_slice(raw).map_err(|e| RpcError::Deser(format!("{}", e)))
}