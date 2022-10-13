use image::{load_from_memory};
use wasmbus_rpc::actor::prelude::*;


/// Preprocesses a given image in order to match requirements for `mobilenet_v1_1.0_224_quantized`
/// 
/// Given that each model has specific requirements regarding its input parameters,
/// it make sense to provide specific preprocessing as well. This prepcrocessor targets
/// Coral's edge TPU. The peculiarity of edge TPU is that is accepts input only in form
/// of `integer` values.
/// 
/// In addition, values usually have to fulfill some statistical requirements. The 
/// requirements for `mobilenet_v1_1.0_224_quantized` are disussed on [tfhub.dev](https://tfhub.dev/tensorflow/coral-model/mobilenet_v1_1.0_224_quantized/1/default/1)
/// Hints about how to preprocess are given in the [pycoral](https://github.com/google-coral/pycoral/blob/master/examples/classify_image.py) repository:
/// 
/// ```python
///  # Image data must go through two transforms before running inference:
///  # 1. normalization: f = (input - mean) / std
///  # 2. quantization: q = f / scale + zero_point
///  # The following code combines the two steps as such:
///  # q = (input - mean) / (std * scale) + zero_point
///  # However, if std * scale equals 1, and mean - zero_point equals 0, the input
///  # does not need any preprocessing (but in practice, even if the results are
///  # very close to 1 and 0, it is probably okay to skip preprocessing for better
///  # efficiency; we use 1e-5 below instead of absolute zero).
/// 
///  params = common.input_details(interpreter, 'quantization_parameters')
///  scale = params['scales']
///  zero_point = params['zero_points']
///  mean = args.input_mean
///  std = args.input_std
///  if abs(scale * std - 1) < 1e-5 and abs(mean - zero_point) < 1e-5:
///    # Input data does not require preprocessing.
///    common.set_input(interpreter, image)
///  else:
///    # Input data requires preprocessing
///    normalized_input = (np.asarray(image) - mean) / (std * scale) + zero_point
///    np.clip(normalized_input, 0, 255, out=normalized_input)
///    common.set_input(interpreter, normalized_input.astype(np.uint8))
/// ``` 
/// 
/// Metadata like `scale` and `zero_point` can be taken from the model. For this model,
/// `scale` == 1/128
/// `zero_point` == 128
/// 
pub async fn preprocess(raw_data: &[u8], height: u32, width: u32) -> RpcResult<Vec<u8>> {
    log::debug!("preprocess() - entry point");

    let raw_image = load_from_memory(raw_data).map_err(|e| RpcError::Deser(e.to_string()))?;

    let image = image::imageops::resize(
        &raw_image.to_rgb8(),
        width,
        height,
        ::image::imageops::FilterType::Triangle,
    );

    Ok(image.into_vec())
}