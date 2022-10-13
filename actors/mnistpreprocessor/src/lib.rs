use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_logging::debug;
use wasmcloud_interface_mlpreprocessing::{
    ConversionOutput, ConversionRequest, MlPreprocessing, MlPreprocessingReceiver, Status, Tensor,
    ValueType,
};

mod img;

#[derive(Debug, Default, Actor, HealthResponder)]
#[services(Actor, MlPreprocessing)]
struct MNistpreprocessorActor {}

/// Implementation of MlPreprocessing trait methods
#[async_trait]
impl MlPreprocessing for MNistpreprocessorActor {
    async fn convert(
        &self,
        _ctx: &Context,
        arg: &ConversionRequest,
    ) -> RpcResult<ConversionOutput> {
        let image: Vec<u8> = arg.data.to_owned();

        // TODO: validate the image, does it have 3 channels? Is it RGB?

        debug!("convert() - BEFORE conversion");

        let convert: Vec<u8> = img::preprocess(&image, 28, 28).await?;

        debug!("convert() - AFTER conversion");

        let t = Tensor {
            value_types: vec![ValueType::ValueF32],
            dimensions: vec![1, 1, 224, 224],
            data: convert,
            flags: 0,
        };

        let convert = ConversionOutput {
            result: Status::Success,
            tensor: t,
        };

        Ok(convert)
    }
}


// use wasmbus_rpc::actor::prelude::*;
// use wasmcloud_interface_httpserver::{HttpRequest, HttpResponse, HttpServer, HttpServerReceiver};

// #[derive(Debug, Default, Actor, HealthResponder)]
// #[services(Actor, HttpServer)]
// struct MnistpreprocessorActor {}

// /// Implementation of HttpServer trait methods
// #[async_trait]
// impl HttpServer for MnistpreprocessorActor {

//     /// Returns a greeting, "Hello World", in the response body.
//     /// If the request contains a query parameter 'name=NAME', the
//     /// response is changed to "Hello NAME"
//     async fn handle_request(
//         &self,
//         _ctx: &Context,
//         req: &HttpRequest,
//     ) -> std::result::Result<HttpResponse, RpcError> {
//         let text = form_urlencoded::parse(req.query_string.as_bytes())
//             .find(|(n, _)| n == "name")
//             .map(|(_, v)| v.to_string())
//             .unwrap_or_else(|| "World".to_string());

//         Ok(HttpResponse {
//             body: format!("Hello {}", text).as_bytes().to_vec(),
//             ..Default::default()
//         })
//     }
// }

