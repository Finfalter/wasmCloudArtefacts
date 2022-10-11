use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_mlpreprocessing::{
    ConversionOutput, ConversionRequest, MlPreprocessing, MlPreprocessingReceiver, Status, Tensor,
    ValueType,
};

mod img;

#[derive(Debug, Default, Actor, HealthResponder)]
#[services(Actor, MlPreprocessing)]
struct ImagenetpreprocessorActor {}

const HEIGHT: u32 = 224;
const WIDTH: u32 = 224;
const CHANNELS: u32 = 3;

/// Implementation of MlPreprocessing trait methods
#[async_trait]
impl MlPreprocessing for ImagenetpreprocessorActor {
    async fn convert(
        &self,
        _ctx: &Context,
        arg: &ConversionRequest,
    ) -> RpcResult<ConversionOutput> {
        let convert: Vec<u8> = img::preprocess(&arg.data, HEIGHT, WIDTH)?;

        let t = Tensor {
            value_types: vec![ValueType::ValueF32],
            dimensions: vec![1, CHANNELS, HEIGHT, WIDTH],
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
