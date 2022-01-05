// mod model;
// use model::SINE_MODEL;

#[path = "../src/utils.rs"]
mod utils;

use utils::{f32_vec_to_bytes};

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

use ndarray::{array};

// In case of problems define the following
// export WASMCLOUD_OCI_ALLOWED_INSECURE=*

const IDENTITY_MODEL_PATH: &str = "tests/identity_input_output.onnx";

async fn get_environment() -> (MlinferenceSender<Provider>, Context) {
    // create a provider, client and context
    let prov = test_provider().await;
    let client = MlinferenceSender::via(prov);
    let ctx = Context::default();

    return (client, ctx);
}

async fn get_load(model_path: &str, e: u8, t: u8) -> LoadInput {
    let graph_builder = std::fs::read(model_path).unwrap();
    let graph_encoding: GraphEncoding = GraphEncoding { encoding: e };
    let execution_target: ExecutionTarget = ExecutionTarget { target: t };

    LoadInput {
        builder: graph_builder,
        encoding: graph_encoding,
        target: execution_target,
    }
}

#[tokio::test]
async fn run_all() {
    let opts = TestOptions::default();
    let res = run_selected_spawn!(
        &opts, 
        health_check, 
        load_one_graph, 
        load_multiple_graphs,
        load_unsupported_encoding,
        init_execution_context_invalid_model,
        init_execution_context_valid_model,
        set_input_happy_path,
        set_input_gec_not_found,
        set_input_corrupt_tensor_input);

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
async fn load_one_graph(_opt: &TestOptions) -> RpcResult<()> {
    let env = get_environment().await;

    //let gb1: GraphBuilder = vec![0xa1, 0xa2, 0xa3];
    //let gb1: GraphBuilder = SINE_MODEL.to_vec();
    let gb1 = std::fs::read(IDENTITY_MODEL_PATH).unwrap();
    let _gb2: GraphBuilder = vec![0xa2, 0xa3, 0xa4];
    let _gb3: GraphBuilder = vec![0xa3, 0xa4, 0xa5];

    let ge: GraphEncoding = GraphEncoding { encoding: 1 };

    let et: ExecutionTarget = ExecutionTarget { target: 0 };

    let load = LoadInput {
        builder: gb1,
        encoding: ge,
        target: et,
    };

    let resp = env.0.load(&env.1, &load).await?;

    assert_eq!(resp.result.has_error, false, "should be: 'false'");
    assert_eq!(resp.result.guest_error, None, "should be 'None'");
    assert_eq!(resp.result.runtime_error, None, "should be 'None'");
    assert_eq!(resp.graph, Graph{graph: 0}, "should be: 0");

    Ok(())
}

/// tests of the Mlinference capability
async fn load_multiple_graphs(_opt: &TestOptions) -> RpcResult<()> {
    let env = get_environment().await;

    let gb1: GraphBuilder = vec![0xa1, 0xa2, 0xa3];
    let gb2: GraphBuilder = vec![0xa2, 0xa3, 0xa4];
    let gb3: GraphBuilder = vec![0xa3, 0xa4, 0xa5];

    let ge: GraphEncoding = GraphEncoding { encoding: 1 };
    let et: ExecutionTarget = ExecutionTarget { target: 0 };

    let load = LoadInput { builder: gb1, encoding: ge.clone(), target: et.clone()};
    let resp = env.0.load(&env.1, &load).await?;
    assert_eq!(resp.graph, Graph{graph: 1}, "should be: 1");

    let load = LoadInput { builder: gb2, encoding: ge.clone(), target: et.clone()};
    let resp = env.0.load(&env.1, &load).await?;
    assert_eq!(resp.graph, Graph{graph: 2}, "should be: 2");

    let load = LoadInput { builder: gb3, encoding: ge, target: et};
    let resp = env.0.load(&env.1, &load).await?;

    assert_eq!(resp.result.has_error, false, "should be: 'false'");
    assert_eq!(resp.result.guest_error, None, "should be 'None'");
    assert_eq!(resp.result.runtime_error, None, "should be 'None'");
    assert_eq!(resp.graph, Graph{graph: 3}, "should be: 3");

    Ok(())
}

/// tests of the Mlinference capability
async fn load_unsupported_encoding(_opt: &TestOptions) -> RpcResult<()> {
    let env = get_environment().await;

    let gb: GraphBuilder = vec![0xa1, 0xa2, 0xa3];

    let ge: GraphEncoding = GraphEncoding     { encoding: 0 }; // 0 := OPENVINO which is currently not supported
    let et: ExecutionTarget = ExecutionTarget { target: 0 };

    let load = LoadInput { builder: gb, encoding: ge.clone(), target: et.clone()};
    let resp = env.0.load(&env.1, &load).await?;

    assert_eq!(resp.result.has_error, true, "should be: 'true'");
    assert_eq!(resp.result.guest_error, Some(GuestError{ model_error: 1}), "should be: 1, corresponding to InvalidEncodingError");
    assert_eq!(resp.result.runtime_error, None, "should be 'None'");
    assert_eq!(resp.graph, Graph{graph: std::u32::MAX}, "should be u32::MAX");

    Ok(())
}

/// tests of the Mlinference capability - init_execution_context()
async fn init_execution_context_valid_model(_opt: &TestOptions) -> RpcResult<()> {
    let env = get_environment().await;

    let resp = env.0.init_execution_context(&env.1, &Graph{graph: 0}).await?;

    assert_eq!(resp.result.has_error, false, "should be: 'false'");
    assert_eq!(resp.result.guest_error, None, "should be '0' for 'None'");
    assert_eq!(resp.result.runtime_error, None, "should be 'None'");
    assert_eq!(resp.gec, GraphExecutionContext{gec: 0}, "should be '0'");

    Ok(())
}

/// tests of the Mlinference capability - init_execution_context()
async fn init_execution_context_invalid_model(_opt: &TestOptions) -> RpcResult<()> {
    let env = get_environment().await;

    let resp = env.0.init_execution_context(&env.1, &Graph{graph: 1}).await?;

    assert_eq!(resp.result.has_error, true, "should be: 'true'");
    assert_eq!(resp.result.guest_error, Some(GuestError{model_error: 0}), "should be '0' for 'ModelError'");
    assert_eq!(resp.result.runtime_error, None, "should be 'None'");
    assert_eq!(resp.gec, GraphExecutionContext{gec: std::u32::MAX}, "should be u32::MAX");

    Ok(())
}

/// tests of the Mlinference capability - set_input()
async fn set_input_happy_path(_opt: &TestOptions) -> RpcResult<()> {
    let env = get_environment().await;

    let load = get_load(
        IDENTITY_MODEL_PATH,
         utils::GraphEncoding::GRAPH_ENCODING_ONNX,
         utils::ExecutionTarget::EXECUTION_TARGET_CPU)
         .await;

    let load_result = env.0.load(&env.1, &load).await?;
    assert_eq!(load_result.result.has_error, false, "should be: 'false'");
    
    let iec_result = env.0.init_execution_context(&env.1, &load_result.graph).await?;
    assert_eq!(iec_result.result.has_error, false, "should be: 'false'");

    let input_tensor = array![[1.0, 2.0, 3.0, 4.0]];
    let shape: Vec<u32> = input_tensor.shape().iter().map(|u| *u as u32).collect();
    println!("set_input_basics() - sending simple tensor: {:#?}", input_tensor);

    let tensor = f32_vec_to_bytes(input_tensor.as_slice().unwrap().to_vec());

    let set_input_struct = SetInputStruct {
        context: iec_result.gec,
        index: Some(0),
        tensor: Tensor {
            dimensions: shape,
            ttype: TensorType{ttype: 1},
            data: tensor,
        }
    };

    let set_result = env.0.set_input(&env.1, &set_input_struct).await?;
    assert_eq!(set_result.has_error, false, "should be: 'false'");

    Ok(())
}

/// tests of the Mlinference capability - set_input()
async fn set_input_gec_not_found(_opt: &TestOptions) -> RpcResult<()> {
    let env = get_environment().await;

    let load = get_load(
        IDENTITY_MODEL_PATH,
         utils::GraphEncoding::GRAPH_ENCODING_ONNX,
         utils::ExecutionTarget::EXECUTION_TARGET_CPU)
         .await;

    let load_result = env.0.load(&env.1, &load).await?;
    assert_eq!(load_result.result.has_error, false, "should be: 'false'");
    
    let iec_result = env.0.init_execution_context(&env.1, &load_result.graph).await?;
    assert_eq!(iec_result.result.has_error, false, "should be: 'false'");

    let input_tensor = array![[1.0, 2.0, 3.0, 4.0]];
    let shape: Vec<u32> = input_tensor.shape().iter().map(|u| *u as u32).collect();
    println!("set_input_basics() - sending simple tensor: {:#?}", input_tensor);

    let tensor = f32_vec_to_bytes(input_tensor.as_slice().unwrap().to_vec());

    let set_input_struct = SetInputStruct {
        context: GraphExecutionContext{gec: std::u32::MAX}, // --> by purpose, this value is invalid
        index: Some(0),
        tensor: Tensor {
            dimensions: shape,
            ttype: TensorType{ttype: 1},
            data: tensor,
        }
    };

    let set_result = env.0.set_input(&env.1, &set_input_struct).await?;
    assert_eq!(set_result.has_error, true, "should be: 'false'");
    assert_eq!(set_result.runtime_error, Some(RuntimeError{runtime_error: 3}), "should be: '3' - 'CONTEXT_NOT_FOUND'");

    Ok(())
}

/// tests of the Mlinference capability - set_input()
async fn set_input_corrupt_tensor_input(_opt: &TestOptions) -> RpcResult<()> {
    let env = get_environment().await;

    let load = get_load(
        IDENTITY_MODEL_PATH,
         utils::GraphEncoding::GRAPH_ENCODING_ONNX,
         utils::ExecutionTarget::EXECUTION_TARGET_CPU)
         .await;

    let load_result = env.0.load(&env.1, &load).await?;
    assert_eq!(load_result.result.has_error, false, "should be: 'false'");
    
    let iec_result = env.0.init_execution_context(&env.1, &load_result.graph).await?;
    assert_eq!(iec_result.result.has_error, false, "should be: 'false'");

    let input_tensor = array![[1.0, 2.0, 3.0, 4.0]];
    let shape: Vec<u32> = input_tensor.shape().iter().map(|u| *u as u32).collect();
    println!("set_input_basics() - sending simple tensor: {:#?}", input_tensor);

    let tensor = vec![];

    let set_input_struct = SetInputStruct {
        context: iec_result.gec,
        index: Some(0),
        tensor: Tensor {
            dimensions: shape,
            ttype: TensorType{ttype: 1},
            data: tensor,
        }
    };

    let set_result = env.0.set_input(&env.1, &set_input_struct).await?;
    assert_eq!(set_result.has_error, true, "should be: 'false'");
    assert_eq!(set_result.guest_error, Some(GuestError{model_error: 2}), "should be: '2' - 'CORRUPT_INPUT_TENSOR'");

    Ok(())
}