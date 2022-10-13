use image::{load_from_memory};
use wasmbus_rpc::actor::prelude::*;

pub async fn preprocess(raw_data: &[u8], height: u32, width: u32) -> RpcResult<Vec<u8>> {
    log::debug!("preprocess() - entry point");

    let raw_image = load_from_memory(raw_data).map_err(|e| RpcError::Deser(e.to_string()))?;

    log::debug!("raw_image color type: {:#?}", raw_image.color());

    let image = image::imageops::resize(
        &raw_image,
        width,
        height,
        ::image::imageops::FilterType::Triangle,
    );

    log::debug!("resized image: {:#?}", image.dimensions());

    Ok(raw_image.into_bytes())
}