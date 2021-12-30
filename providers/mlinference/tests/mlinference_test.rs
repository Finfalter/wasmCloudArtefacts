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

// In case of problems define the following
// export WASMCLOUD_OCI_ALLOWED_INSECURE=*

#[tokio::test]
async fn run_all() {
    let opts = TestOptions::default();
    let res = run_selected_spawn!(
        &opts, 
        health_check, 
        load_basics, 
        load_multiple_graphs,
        load_unsupported_encoding);

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

/// tests of the Mlinference capability
async fn factorial_0_1(_opt: &TestOptions) -> RpcResult<()> {
    let prov = test_provider().await;

    // create client and ctx
    let client = MlinferenceSender::via(prov);
    let ctx = Context::default();

    let resp = client.calculate(&ctx, &0).await?;
    assert_eq!(resp, 1, "0!");

    let resp = client.calculate(&ctx, &1).await?;
    assert_eq!(resp, 1, "1!");

    Ok(())
}

/// more tests of the Mlinference interface
async fn factorial_more(_opt: &TestOptions) -> RpcResult<()> {
    let prov = test_provider().await;

    // create client and ctx
    let client = MlinferenceSender::via(prov);
    let ctx = Context::default();

    let resp = client.calculate(&ctx, &2).await?;
    assert_eq!(resp, 2, "2!");

    let resp = client.calculate(&ctx, &3).await?;
    assert_eq!(resp, 6, "3!");

    let resp = client.calculate(&ctx, &4).await?;
    assert_eq!(resp, 24, "4!");

    Ok(())
}

/// tests of the Mlinference capability
async fn load_basics(_opt: &TestOptions) -> RpcResult<()> {
    let prov = test_provider().await;

    // create client and ctx
    let client = MlinferenceSender::via(prov);
    let ctx = Context::default();

    let gb1: GraphBuilder = vec![0xa1, 0xa2, 0xa3];
    let _gb2: GraphBuilder = vec![0xa2, 0xa3, 0xa4];
    let _gb3: GraphBuilder = vec![0xa3, 0xa4, 0xa5];

    let ge: GraphEncoding = GraphEncoding { encoding: 1 };

    let et: ExecutionTarget = ExecutionTarget { target: 0 };

    let load = LoadInput {
        builder: gb1,
        encoding: ge,
        target: et,
    };

    let resp = client.load(&ctx, &load).await?;

    assert_eq!(resp.has_error, false, "should be: 'false'");
    assert_eq!(resp.guest_error, None, "should be 'None'");
    assert_eq!(resp.runtime_error, None, "should be 'None'");
    assert_eq!(resp.graph, Graph{graph: 0}, "should be: 0");

    Ok(())
}

/// tests of the Mlinference capability
async fn load_multiple_graphs(_opt: &TestOptions) -> RpcResult<()> {
    let prov = test_provider().await;

    // create client and ctx
    let client = MlinferenceSender::via(prov);
    let ctx = Context::default();

    let gb1: GraphBuilder = vec![0xa1, 0xa2, 0xa3];
    let gb2: GraphBuilder = vec![0xa2, 0xa3, 0xa4];
    let gb3: GraphBuilder = vec![0xa3, 0xa4, 0xa5];

    let ge: GraphEncoding = GraphEncoding { encoding: 1 };
    let et: ExecutionTarget = ExecutionTarget { target: 0 };

    let load = LoadInput { builder: gb1, encoding: ge.clone(), target: et.clone()};
    let resp = client.load(&ctx, &load).await?;
    assert_eq!(resp.graph, Graph{graph: 1}, "should be: 1");

    let load = LoadInput { builder: gb2, encoding: ge.clone(), target: et.clone()};
    let resp = client.load(&ctx, &load).await?;
    assert_eq!(resp.graph, Graph{graph: 2}, "should be: 2");

    let load = LoadInput { builder: gb3, encoding: ge, target: et};
    let resp = client.load(&ctx, &load).await?;

    assert_eq!(resp.has_error, false, "should be: 'false'");
    assert_eq!(resp.guest_error, None, "should be 'None'");
    assert_eq!(resp.runtime_error, None, "should be 'None'");
    assert_eq!(resp.graph, Graph{graph: 3}, "should be: 3");

    Ok(())
}

/// tests of the Mlinference capability
async fn load_unsupported_encoding(_opt: &TestOptions) -> RpcResult<()> {
    let prov = test_provider().await;

    // create client and ctx
    let client = MlinferenceSender::via(prov);
    let ctx = Context::default();

    let gb: GraphBuilder = vec![0xa1, 0xa2, 0xa3];

    let ge: GraphEncoding = GraphEncoding     { encoding: 0 }; // 0 := OPENVINO which is currently not supported
    let et: ExecutionTarget = ExecutionTarget { target: 0 };

    let load = LoadInput { builder: gb, encoding: ge.clone(), target: et.clone()};
    let resp = client.load(&ctx, &load).await?;

    assert_eq!(resp.has_error, true, "should be: 'true'");
    assert_eq!(resp.guest_error, Some(GuestError{ model_error: 1}), "should be: 1, corresponding to InvalidEncodingError");
    assert_eq!(resp.runtime_error, None, "should be 'None'");
    assert_eq!(resp.graph, Graph{graph: std::u32::MAX}, "should be: u32::MAX");

    Ok(())
}