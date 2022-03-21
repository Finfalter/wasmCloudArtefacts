use ndarray::array;
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

use wasmcloud_provider_mlinference::inference::f32_vec_to_bytes;

async fn get_environment() -> (MlinferenceSender<Provider>, Context) {
    // create a provider, client and context
    let prov = test_provider().await;

    eprintln!("Pausing 5 sec to load model");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let client = MlinferenceSender::via(prov);
    let ctx = Context::default();

    (client, ctx)
}

#[tokio::test]
async fn run_all() {
    let opts = TestOptions::default();
    let res = run_selected_spawn!(&opts, health_check, test_one);
    print_test_results(&res);

    let passed = res.iter().filter(|tr| tr.passed).count();
    let total = res.len();
    assert_eq!(passed, total, "{} passed out of {}", passed, total);

    // try to let the provider shut dowwn gracefully
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

/// more tests of the Mlinference interface
async fn test_one(_opt: &TestOptions) -> RpcResult<()> {
    let env = get_environment().await;

    let input_tensor = array![[1.0, 2.0, 3.0, 4.0]];
    println!("input_tensor: {:#?}", input_tensor);

    let tensor_shape: Vec<u32> = input_tensor.shape().iter().map(|u| *u as u32).collect();
    println!("shape: {:#?}", tensor_shape);

    let tensor_data = f32_vec_to_bytes(input_tensor.as_slice().unwrap().to_vec());
    println!("input_tensor: {:#?}", input_tensor);

    let tensor_data_cloned = tensor_data.clone();
    let tensor_shape_cloned = tensor_shape.clone();

    let t = Tensor {
        tensor_type: TensorType { ttype: 0 },
        dimensions: tensor_shape,
        data: tensor_data,
    };

    let ir = InferenceRequest {
        model: "identity".to_string(),
        tensor: t,
        index: 0,
    };

    let predict_result = env.0.predict(&env.1, &ir).await?;

    println!("test_one() with result {:?}", predict_result);

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
