use image::Pixel;
use anyhow::Error;
use std::{fmt::Debug, io::Cursor};
use ndarray::{s, Array, ArrayBase};
use wasmcloud_provider_mlinference::inference::f32_array_to_bytes;

pub fn image_to_tensor<S: Into<String> + AsRef<std::path::Path> + Debug>(
    path: S,
    height: u32,
    width: u32,
) -> Result<Vec<u8>, Error> {
    println!("trying to load image {:#?}", path);
    let image = image::imageops::resize(
        &image::open(path)?,
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

    Ok(f32_vec_to_bytes(array.as_slice().unwrap()))
}