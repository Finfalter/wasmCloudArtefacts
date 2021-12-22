//! mlinference capability provider
//!
//!
use log::debug;
use wasmbus_rpc::provider::prelude::*;
use wasmcloud_interface_mlinference::{
    Mlinference, MlinferenceReceiver, LoadInput, Graph};

// main (via provider_main) initializes the threaded tokio executor,
// listens to lattice rpcs, handles actor links,
// and returns only when it receives a shutdown message
//
fn main() -> Result<(), Box<dyn std::error::Error>> {
    provider_main(MlinferenceProvider::default())?;

    eprintln!("mlinference provider exiting");
    Ok(())
}

/// mlinference capability provider implementation
#[derive(Default, Clone, Provider)]
#[services(Mlinference)]
struct MlinferenceProvider {}

/// use default implementations of provider message handlers
impl ProviderDispatch for MlinferenceProvider {}
impl ProviderHandler for MlinferenceProvider {}

/// Handle Mlinference methods
#[async_trait]
impl Mlinference for MlinferenceProvider {
    /// accepts a number and calculates its factorial
    async fn calculate(&self, _ctx: &Context, req: &u32) -> RpcResult<u64> {
        debug!("processing request calculate({})", *req);
        Ok(n_factorial(*req))
    }

    /// load
    async fn load(&self, _ctx: &Context, _arg: &LoadInput) -> RpcResult<Graph>{
        debug!("processing request load()");
        Ok(Graph {graph: 42})
    }
}

/// calculate n factorial
fn n_factorial(n: u32) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        _ => {
            let mut result = 1u64;
            // add 1 because rust ranges exclude upper bound
            for v in 2..(n + 1) {
                result *= v as u64;
            }
            result
        }
    }
}
