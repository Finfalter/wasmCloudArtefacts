use wasmbus_rpc::provider::prelude::*;
use wasmcloud_interface_mlinference::*;
use wasmcloud_test_util::{
    check,
    cli::print_test_results,
    provider_test::test_provider,
    testing::{TestOptions, TestResult},
};
#[allow(unused_imports)]
use wasmcloud_test_util::{run_selected, run_selected_spawn};

const IDENTITY_MODEL_PATH: &str = "tests/testdata/models/identity_input_output.onnx";

#[tokio::test]
async fn run_all() {
    let opts = TestOptions::default();
    //let res = run_selected_spawn!(&opts, health_check, test_one);
    let res = run_selected_spawn!(&opts, test_one);
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
    let prov = test_provider().await;
    let ctx = Context::default();

    let client = MlinferenceSender::via(prov);

    let model = std::fs::read(IDENTITY_MODEL_PATH).unwrap();

    let t = Tensor {
        ttype: TensorType { ttype: 0},
        dimensions: vec![1, 2, 3, 4],
        data: model
    };

    let ir = InferenceRequest {
        model: "flex".to_string(),
        tensor: t,
        index: 0
    };

    let resp = client.predict(&ctx, &ir).await?;

    Ok(())
}
