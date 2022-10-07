use image::{load_from_memory};
use wasmbus_rpc::actor::prelude::*;
//use wasmcloud_interface_logging::debug;

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