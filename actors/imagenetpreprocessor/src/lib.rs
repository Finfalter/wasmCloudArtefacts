use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_logging::debug;
use wasmcloud_interface_mlpreprocessing::{
    ConversionOutput, ConversionRequest, MlPreprocessing, MlPreprocessingReceiver, Status, Tensor,
    ValueType,
};

mod img;

#[derive(Debug, Default, Actor, HealthResponder)]
#[services(Actor, MlPreprocessing)]
struct ImagenetpreprocessorActor {}

/// Implementation of MlPreprocessing trait methods
#[async_trait]
impl MlPreprocessing for ImagenetpreprocessorActor {
    async fn convert(
        &self,
        _ctx: &Context,
        arg: &ConversionRequest,
    ) -> RpcResult<ConversionOutput> {
        let image: Vec<u8> = arg.data.to_owned();

        let height = 224;
        let width  = 224;
        let channels = 3;

        // TODO: validate the image, does it have 3 channels? Is it RGB?

        debug!("convert() - BEFORE conversion");

        let convert: Vec<u8> = img::preprocess(&image, height, width).await?;

        debug!("convert() - AFTER conversion");

        let t = Tensor {
            value_types: vec![ValueType::ValueF32],
            dimensions: vec![1, channels, height, width],
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
