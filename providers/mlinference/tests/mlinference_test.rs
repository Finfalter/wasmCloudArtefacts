use anyhow::Error;
use image::Pixel;
use ndarray::{array, s, Array, ArrayBase};
use std::{
    fmt::Debug,
    io::{BufRead, BufReader},
};
use wasmbus_rpc::provider::prelude::*;
use wasmcloud_interface_mlinference::*;
use wasmcloud_test_util::{
    check,
    cli::print_test_results,
    provider_test::{test_provider, Provider},
    testing::{TestOptions, TestResult},
};
#[allow(unused_imports)]
use wasmcloud_test_util::{run_selected, run_selected_spawn};

use wasmcloud_provider_mlinference::inference::{bytes_to_f32_vec, f32_array_to_bytes};

pub trait NdArrayTensor<S, T, D> {
    /// https://en.wikipedia.org/wiki/Softmax_function
    fn softmax(&self, axis: ndarray::Axis) -> Array<T, D>
    where
        D: ndarray::RemoveAxis,
        S: ndarray::RawData + ndarray::Data + ndarray::RawData<Elem = T>,
        <S as ndarray::RawData>::Elem: std::clone::Clone,
        T: ndarray::NdFloat + std::ops::SubAssign + std::ops::DivAssign;
}

impl<S, T, D> NdArrayTensor<S, T, D> for ArrayBase<S, D>
where
    D: ndarray::RemoveAxis,
    S: ndarray::RawData + ndarray::Data + ndarray::RawData<Elem = T>,
    <S as ndarray::RawData>::Elem: std::clone::Clone,
    T: ndarray::NdFloat + std::ops::SubAssign + std::ops::DivAssign,
{
    fn softmax(&self, axis: ndarray::Axis) -> Array<T, D> {
        let mut new_array: Array<T, D> = self.to_owned();
        new_array.map_inplace(|v| *v = v.exp());
        let sum = new_array.sum_axis(axis).insert_axis(axis);
        new_array /= &sum;

        new_array
    }
}

pub async fn image_to_tensor<S: Into<String> + AsRef<std::path::Path> + Debug>(
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

    Ok(f32_array_to_bytes(array.as_slice().unwrap()).await)
}

async fn get_environment() -> (MlInferenceSender<Provider>, Context) {
    // create a provider, client and context
    let prov = test_provider().await;

    eprintln!("Pausing 5 sec to load model");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let client = MlInferenceSender::via(prov);
    let ctx = Context::default();

    (client, ctx)
}

const IMG_PATH: &str = "tests/testdata/images/n04350905.jpg";
const LABELS_PATH: &str = "tests/testdata/models/squeezenet_labels.txt";

#[tokio::test]
async fn run_all() {
    let opts = TestOptions::default();

    let res = run_selected_spawn!(
        opts,
        health_check,
        onnx_identity_input_output,
        tensorflow_plus3,
        onnx_mobilenetv2_7,
        onnx_squeezenetv1_1_7
    );

    print_test_results(&res);

    let passed = res.iter().filter(|tr| tr.passed).count();
    let total = res.len();
    assert_eq!(passed, total, "{} passed out of {}", passed, total);

    // try to let the provider shut down gracefully
    let provider = test_provider().await;
    let _ = provider.shutdown().await;
}

/// test that health check returns healthy
async fn health_check(_opt: &TestOptions) -> RpcResult<()> {
    let prov = test_provider().await;

    // health check
    let hc = prov.health_check().await;
    check!(hc.is_ok())?;
    Ok(())
}

/// testing ONNX inference engine with model 'identity_input_output'
async fn onnx_identity_input_output(_opt: &TestOptions) -> RpcResult<()> {
    let env = get_environment().await;

    let input_tensor = array![[1.0, 2.0, 3.0, 4.0]];
    println!("input_tensor: {:#?}", input_tensor);

    let tensor_shape: Vec<u32> = input_tensor.shape().iter().map(|u| *u as u32).collect();
    println!("shape: {:#?}", tensor_shape);

    let tensor_data = f32_array_to_bytes(input_tensor.as_slice().unwrap()).await;
    println!("input_tensor: {:#?}", input_tensor);

    let tensor_data_cloned = tensor_data.clone();
    let tensor_shape_cloned = tensor_shape.clone();

    let t = Tensor {
        value_types: vec![ValueType::ValueF32],
        dimensions: tensor_shape,
        data: tensor_data,
        flags: 0,
    };

    let ir = InferenceInput {
        model: "identity".to_string(),
        tensor: t,
        index: 0,
    };

    let predict_result = env.0.predict(&env.1, &ir).await?;

    println!(
        "onnx_identity_input_output() with result {:?}",
        predict_result
    );

    assert_eq!(
        tensor_data_cloned, predict_result.tensor.data,
        "Output data should be the same as input data"
    );
    assert_eq!(
        tensor_shape_cloned, predict_result.tensor.dimensions,
        "Output shape should be the same as input shape"
    );

    Ok(())
}

/// testing Tensorflow inference engine with model 'plus3'
async fn tensorflow_plus3(_opt: &TestOptions) -> RpcResult<()> {
    let env = get_environment().await;

    let input_tensor = array![[1.0, 2.0, 3.0, 4.0]];

    // expected result after being processed by model 'plus 3'
    let output_tensor = array![[4.0, 5.0, 6.0, 7.0]];

    println!("input_tensor: {:#?}", input_tensor);

    let tensor_shape: Vec<u32> = input_tensor.shape().iter().map(|u| *u as u32).collect();
    println!("shape: {:#?}", tensor_shape);

    let tensor_data = f32_array_to_bytes(input_tensor.as_slice().unwrap()).await;
    println!("input_tensor: {:#?}", input_tensor);

    let tensor_shape_cloned = tensor_shape.clone();

    let t = Tensor {
        value_types: vec![ValueType::ValueF32],
        dimensions: tensor_shape,
        data: tensor_data,
        flags: 0,
    };

    let ir = InferenceInput {
        model: "plus3".to_string(),
        tensor: t,
        index: 0,
    };

    let predict_result = env.0.predict(&env.1, &ir).await?;

    println!("tensorflow_plus3() with result {:?}", predict_result);

    assert_eq!(
        predict_result.tensor.data,
        f32_array_to_bytes(output_tensor.as_slice().unwrap()).await,
        "Output data should be input 'plus 3'"
    );
    assert_eq!(
        tensor_shape_cloned, predict_result.tensor.dimensions,
        "Output shape should be the same as input shape"
    );

    Ok(())
}

/// testing ONNX inference engine with model 'mobilenetv2-7'
async fn onnx_mobilenetv2_7(_opt: &TestOptions) -> RpcResult<()> {
    let env = get_environment().await;

    let image = image_to_tensor(IMG_PATH, 224, 224).await.unwrap();

    //println!("input_tensor: {:#?}", input_tensor);

    let t = Tensor {
        value_types: vec![ValueType::ValueF32],
        dimensions: vec![1, 3, 224, 224],
        data: image,
        flags: 0,
    };

    let ir = InferenceInput {
        model: "mobilenetv27".to_string(),
        tensor: t,
        index: 0,
    };

    let raw_result_bytes = env.0.predict(&env.1, &ir).await?;

    //println!("ONNX-mobilenetv27() with result {:?}", raw_result);

    // TODO: assert that there is no error
    let raw_result_f32 = bytes_to_f32_vec(raw_result_bytes.tensor.data)
        .await
        .unwrap();

    let output_tensor = Array::from_shape_vec((1, 1000, 1, 1), raw_result_f32).unwrap();
    let mut probabilities: Vec<(usize, f32)> = output_tensor
        .softmax(ndarray::Axis(1))
        .into_iter()
        .enumerate()
        .collect::<Vec<_>>();
    probabilities.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let labels = BufReader::new(std::fs::File::open(LABELS_PATH).unwrap());

    let mut actual: Vec<String> = Vec::new();
    let labels: Vec<String> = labels.lines().map(|line| line.unwrap()).collect();

    println!("results for image {:#?}", IMG_PATH);

    for i in 0..5 {
        let c = labels[probabilities[i].0].clone();
        actual.push(c);
        println!(
            "class={} ({}); probability={}",
            labels[probabilities[i].0], probabilities[i].0, probabilities[i].1
        );
    }

    log::debug!("THIS IS A TEST - BUT NOT VISIBLE! --------------------------------------- ");

    // assert_eq!(
    //     predict_result.tensor.data, f32_vec_to_bytes(output_tensor.as_slice().unwrap().to_vec()),
    //     "Output data should be input 'plus 3'"
    // );
    // assert_eq!(
    //     tensor_shape_cloned, predict_result.tensor.dimensions,
    //     "Output shape should be the same as input shape"
    // );

    Ok(())
}

/// testing ONNX inference engine with model 'mobilenetv2-7'
async fn onnx_squeezenetv1_1_7(_opt: &TestOptions) -> RpcResult<()> {
    let env = get_environment().await;

    let image = image_to_tensor(IMG_PATH, 224, 224).await.unwrap();

    //println!("input_tensor: {:#?}", input_tensor);

    let t = Tensor {
        value_types: vec![ValueType::ValueF32],
        dimensions: vec![1, 3, 224, 224],
        data: image,
        flags: 0,
    };

    let ir = InferenceInput {
        model: "squeezenetv117".to_string(),
        tensor: t,
        index: 0,
    };

    let raw_result_bytes = env.0.predict(&env.1, &ir).await?;

    //println!("ONNX-mobilenetv27() with result {:?}", raw_result);

    // TODO: assert that there is no error
    let raw_result_f32 = bytes_to_f32_vec(raw_result_bytes.tensor.data)
        .await
        .unwrap();

    let output_tensor = Array::from_shape_vec((1, 1000, 1, 1), raw_result_f32).unwrap();
    let mut probabilities: Vec<(usize, f32)> = output_tensor
        .softmax(ndarray::Axis(1))
        .into_iter()
        .enumerate()
        .collect::<Vec<_>>();

    probabilities.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let labels = BufReader::new(std::fs::File::open(LABELS_PATH).unwrap());

    let mut actual: Vec<String> = Vec::new();
    let labels: Vec<String> = labels.lines().map(|line| line.unwrap()).collect();

    println!("results for image {:#?}", IMG_PATH);

    for i in 0..5 {
        let c = labels[probabilities[i].0].clone();
        actual.push(c);
        println!(
            "class={} ({}); probability={}",
            labels[probabilities[i].0], probabilities[i].0, probabilities[i].1
        );
    }

    log::debug!("WHY CANT I SEE THIS OUTPUT? --------------------------------------- ");
    eprintln!("PERHAPS THIS?");

    Ok(())
}
