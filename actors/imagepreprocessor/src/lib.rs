use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_mlpreprocessing::{
    MlPreprocessing, MlPreprocessingReceiver, Status,
    ConversionOutput, ConversionRequest, Tensor, ValueType};

mod img;

#[derive(Debug, Default, Actor, HealthResponder)]
#[services(Actor, MlPreprocessing)]
struct ImagepreprocessorActor {}

/// Implementation of MlPreprocessing trait methods
#[async_trait]
impl MlPreprocessing for ImagepreprocessorActor {

    async fn convert(&self, _ctx: &Context, arg: &ConversionRequest) 
    -> RpcResult<ConversionOutput>
    {
        let image: Vec<u8> = arg.data.to_owned();

        // TODO: validate the image, does it have 3 channels? Is it RGB?

        let convert: Vec<u8> = img::preprocess(&image, 244, 244).await?;

        let t = Tensor {
            value_types: vec![ ValueType::ValueF32 ],
            dimensions: vec![1, 3, 224, 224],
            data: convert,
            flags: 0,
        };
        
        let convert = ConversionOutput {
            result: Status::Success,
            tensor: t
        };

        Ok(convert)
    }
}

