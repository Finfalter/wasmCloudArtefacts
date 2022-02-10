use wasmbus_rpc::provider::prelude::*;
use wasmcloud_interface_mlinference::*;
use wasmcloud_test_util::{
    check,
    cli::print_test_results,
    provider_test::{test_provider,Provider},
    testing::{TestOptions, TestResult},
};
#[allow(unused_imports)]
use wasmcloud_test_util::{run_selected, run_selected_spawn};

const IDENTITY_MODEL_PATH: &str = "tests/testdata/models/identity_input_output.onnx";

async fn get_environment() -> (MlinferenceSender<Provider>, Context) {
    // create a provider, client and context
    let prov = test_provider().await;
    let client = MlinferenceSender::via(prov);
    let ctx = Context::default();

    return (client, ctx);
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

    log::info!("==============> test_one()");

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

    let resp = env.0.predict(&env.1, &ir).await?;

    log::debug!("test_one() with result {:?}", resp);

    log::info!("test_one() ==============>");

    Ok(())
}
