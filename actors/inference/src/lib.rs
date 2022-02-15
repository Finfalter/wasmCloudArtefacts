use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_mlcompute::{Converter, ComputeRequest, ComputeOutput, ResultStatus};
use wasmcloud_interface_mlinference::{Mlinference, MlinferenceSender, InferenceOutput, InferenceRequest};
use wasmcloud_interface_mlcompute::Tensor as CTensor;

#[derive(Debug, Default, Actor, HealthResponder)]
#[services(Actor)]
struct InferenceActor {}

#[async_trait]
impl Converter for InferenceActor {

    /// Compute
    async fn compute(&self, ctx: &Context, arg: &ComputeRequest) -> RpcResult<ComputeOutput>
    {      
        let tensor_type:u8 = arg.tensor.ttype.ttype;
        let model = arg.model.to_string();
        let dimensions = arg.tensor.dimensions.clone();
        let data = arg.tensor.data.clone();

        // TODO: validate tensor_type, once it is a union

        if model.is_empty() || dimensions.is_empty() || data.is_empty() {
            return Err(RpcError::InvalidParameter(
                "all of the following must not be empty: [model name; dimensions; data]"
                .to_string()
            ));
        }

        let ir = InferenceRequest {
            model: model,
            tensor: wasmcloud_interface_mlinference::Tensor{ 
                ttype: wasmcloud_interface_mlinference::TensorType{ ttype: tensor_type},
                dimensions: dimensions,
                data: data},
            index: arg.index
        };

        let provider = MlinferenceSender::new();

        let inference_out: InferenceOutput = provider.predict(ctx, &ir).await
            .map_err(|e| RpcError::HostError(e.to_string()))?;

        if inference_out.result.has_error {
            return Ok(
                ComputeOutput {
                    result: ResultStatus::default(),
                    tensor: wasmcloud_interface_mlcompute::Tensor::default()
                }
            );
        };

        let compute_tensor = inference_out.tensor;

        let co: ComputeOutput = ComputeOutput {
            result: ResultStatus::default(),
            tensor: CTensor{ data: compute_tensor.data, dimensions: compute_tensor.dimensions, ..Default::default() },
        };

        Ok(co)
    }
}