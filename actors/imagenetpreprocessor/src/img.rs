use image::{load_from_memory, Pixel};
use ndarray::s;
use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_logging::debug;

pub async fn f32_vec_to_bytes(float_array: Vec<f32>) -> RpcResult<Vec<u8>> {
    let sum: f32 = float_array.iter().sum();

    debug!(
        "f32_vec_to_bytes() - flattened tensor contains {} elements with sum {}",
        float_array.len(),
        sum
    );

    let chunks: Vec<[u8; 4]> = float_array.into_iter().map(|f| f.to_le_bytes()).collect();
    let byte_array: Vec<u8> = chunks.iter().flatten().copied().collect();

    debug!(
        "f32_vec_to_bytes() - flattened byte tensor contains {} elements",
        byte_array.len()
    );

    Ok(byte_array)
}

pub async fn preprocess(raw_data: &[u8], height: u32, width: u32) -> RpcResult<Vec<u8>> {
    log::debug!("preprocess() - HERE");

    let raw_image = load_from_memory(raw_data).map_err(|e| RpcError::Deser(e.to_string()))?;

    let image = image::imageops::resize(
        &raw_image,
        width,
        height,
        ::image::imageops::FilterType::Triangle,
    );

    println!("resized image: {:#?}", image.dimensions());

    let mut array = ndarray::Array::from_shape_fn((1, 3, 224, 224), |(_, c, j, i)| {
        let pixel = image.get_pixel(i as u32, j as u32);
        let channels = pixel.channels();

        // range [0, 255] -> range [0, 1]
        (channels[c] as f32) / 255.0
    });

    // Normalize channels to mean=[0.485, 0.456, 0.406] and std=[0.229, 0.224, 0.225]
    let mean = [0.485, 0.456, 0.406];
    let std = [0.229, 0.224, 0.225];
    for c in 0..3 {
        let mut channel_array = array.slice_mut(s![0, c, .., ..]);
        channel_array -= mean[c];
        channel_array /= std[c];
    }

    let flattened_img: Vec<u8> = f32_vec_to_bytes(array.as_slice().unwrap().to_vec()).await?;

    Ok(flattened_img)
}
