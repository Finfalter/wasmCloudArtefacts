// This file is generated automatically using wasmcloud/weld-codegen and smithy model definitions
//

#![allow(unused_imports, clippy::ptr_arg, clippy::needless_lifetimes)]
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, io::Write, string::ToString};
use wasmbus_rpc::{
    deserialize, serialize, Context, Message, MessageDispatch, RpcError, RpcResult, SendOpts,
    Timestamp, Transport,
};

pub const SMITHY_VERSION: &str = "1.0";

/// BaseResult
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct BaseResult {
    #[serde(rename = "guestError")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guest_error: Option<GuestError>,
    #[serde(rename = "hasError")]
    #[serde(default)]
    pub has_error: bool,
    #[serde(rename = "runtimeError")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_error: Option<RuntimeError>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ExecutionTarget {
    pub target: u8,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Graph {
    pub graph: u32,
}

/// see LoadInput
pub type GraphBuilder = Vec<u8>;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct GraphEncoding {
    pub encoding: u8,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct GraphExecutionContext {
    pub gec: u32,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct GuestError {
    #[serde(rename = "modelError")]
    pub model_error: u8,
}

/// InitExecutionContextResult
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct IecResult {
    pub gec: GraphExecutionContext,
    pub result: BaseResult,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct LoadInput {
    pub builder: GraphBuilder,
    pub encoding: GraphEncoding,
    pub target: ExecutionTarget,
}

/// LoadResult
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct LoadResult {
    pub graph: Graph,
    pub result: BaseResult,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct RuntimeError {
    #[serde(rename = "runtimeError")]
    pub runtime_error: u8,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct SetInputStruct {
    pub context: GraphExecutionContext,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<u32>,
    pub tensor: Tensor,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Tensor {
    pub data: TensorData,
    pub dimensions: TensorDimensions,
    pub ttype: TensorType,
}

pub type TensorData = Vec<u8>;

pub type TensorDimensions = Vec<u32>;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct TensorType {
    pub ttype: u8,
}

/// The Mlinference service
/// wasmbus.contractId: example:interfaces:mlinference
/// wasmbus.providerReceive
/// wasmbus.actorReceive
#[async_trait]
pub trait Mlinference {
    /// returns the capability contract id for this interface
    fn contract_id() -> &'static str {
        "example:interfaces:mlinference"
    }
    /// load
    async fn load(&self, ctx: &Context, arg: &LoadInput) -> RpcResult<LoadResult>;
    /// init_execution_context
    async fn init_execution_context(&self, ctx: &Context, arg: &Graph) -> RpcResult<IecResult>;
    /// set_input
    async fn set_input(&self, ctx: &Context, arg: &SetInputStruct) -> RpcResult<BaseResult>;
    /// compute
    async fn compute(&self, ctx: &Context, arg: &GraphExecutionContext) -> RpcResult<BaseResult>;
}

/// MlinferenceReceiver receives messages defined in the Mlinference service trait
/// The Mlinference service
#[doc(hidden)]
#[async_trait]
pub trait MlinferenceReceiver: MessageDispatch + Mlinference {
    async fn dispatch(&self, ctx: &Context, message: &Message<'_>) -> RpcResult<Message<'_>> {
        match message.method {
            "Load" => {
                let value: LoadInput = deserialize(message.arg.as_ref())
                    .map_err(|e| RpcError::Deser(format!("message '{}': {}", message.method, e)))?;
                let resp = Mlinference::load(self, ctx, &value).await?;
                let buf = serialize(&resp)?;
                Ok(Message {
                    method: "Mlinference.Load",
                    arg: Cow::Owned(buf),
                })
            }
            "InitExecutionContext" => {
                let value: Graph = deserialize(message.arg.as_ref())
                    .map_err(|e| RpcError::Deser(format!("message '{}': {}", message.method, e)))?;
                let resp = Mlinference::init_execution_context(self, ctx, &value).await?;
                let buf = serialize(&resp)?;
                Ok(Message {
                    method: "Mlinference.InitExecutionContext",
                    arg: Cow::Owned(buf),
                })
            }
            "SetInput" => {
                let value: SetInputStruct = deserialize(message.arg.as_ref())
                    .map_err(|e| RpcError::Deser(format!("message '{}': {}", message.method, e)))?;
                let resp = Mlinference::set_input(self, ctx, &value).await?;
                let buf = serialize(&resp)?;
                Ok(Message {
                    method: "Mlinference.SetInput",
                    arg: Cow::Owned(buf),
                })
            }
            "Compute" => {
                let value: GraphExecutionContext = deserialize(message.arg.as_ref())
                    .map_err(|e| RpcError::Deser(format!("message '{}': {}", message.method, e)))?;
                let resp = Mlinference::compute(self, ctx, &value).await?;
                let buf = serialize(&resp)?;
                Ok(Message {
                    method: "Mlinference.Compute",
                    arg: Cow::Owned(buf),
                })
            }
            _ => Err(RpcError::MethodNotHandled(format!(
                "Mlinference::{}",
                message.method
            ))),
        }
    }
}

/// MlinferenceSender sends messages to a Mlinference service
/// The Mlinference service
/// client for sending Mlinference messages
#[derive(Debug)]
pub struct MlinferenceSender<T: Transport> {
    transport: T,
}

impl<T: Transport> MlinferenceSender<T> {
    /// Constructs a MlinferenceSender with the specified transport
    pub fn via(transport: T) -> Self {
        Self { transport }
    }

    pub fn set_timeout(&self, interval: std::time::Duration) {
        self.transport.set_timeout(interval);
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<'send> MlinferenceSender<wasmbus_rpc::provider::ProviderTransport<'send>> {
    /// Constructs a Sender using an actor's LinkDefinition,
    /// Uses the provider's HostBridge for rpc
    pub fn for_actor(ld: &'send wasmbus_rpc::core::LinkDefinition) -> Self {
        Self {
            transport: wasmbus_rpc::provider::ProviderTransport::new(ld, None),
        }
    }
}
#[cfg(target_arch = "wasm32")]
impl MlinferenceSender<wasmbus_rpc::actor::prelude::WasmHost> {
    /// Constructs a client for actor-to-actor messaging
    /// using the recipient actor's public key
    pub fn to_actor(actor_id: &str) -> Self {
        let transport =
            wasmbus_rpc::actor::prelude::WasmHost::to_actor(actor_id.to_string()).unwrap();
        Self { transport }
    }
}

#[cfg(target_arch = "wasm32")]
impl MlinferenceSender<wasmbus_rpc::actor::prelude::WasmHost> {
    /// Constructs a client for sending to a Mlinference provider
    /// implementing the 'example:interfaces:mlinference' capability contract, with the "default" link
    pub fn new() -> Self {
        let transport = wasmbus_rpc::actor::prelude::WasmHost::to_provider(
            "example:interfaces:mlinference",
            "default",
        )
        .unwrap();
        Self { transport }
    }

    /// Constructs a client for sending to a Mlinference provider
    /// implementing the 'example:interfaces:mlinference' capability contract, with the specified link name
    pub fn new_with_link(link_name: &str) -> wasmbus_rpc::RpcResult<Self> {
        let transport = wasmbus_rpc::actor::prelude::WasmHost::to_provider(
            "example:interfaces:mlinference",
            link_name,
        )?;
        Ok(Self { transport })
    }
}
#[async_trait]
impl<T: Transport + std::marker::Sync + std::marker::Send> Mlinference for MlinferenceSender<T> {
    #[allow(unused)]
    /// load
    async fn load(&self, ctx: &Context, arg: &LoadInput) -> RpcResult<LoadResult> {
        let buf = serialize(arg)?;
        let resp = self
            .transport
            .send(
                ctx,
                Message {
                    method: "Mlinference.Load",
                    arg: Cow::Borrowed(&buf),
                },
                None,
            )
            .await?;
        let value = deserialize(&resp)
            .map_err(|e| RpcError::Deser(format!("response to {}: {}", "Load", e)))?;
        Ok(value)
    }
    #[allow(unused)]
    /// init_execution_context
    async fn init_execution_context(&self, ctx: &Context, arg: &Graph) -> RpcResult<IecResult> {
        let buf = serialize(arg)?;
        let resp = self
            .transport
            .send(
                ctx,
                Message {
                    method: "Mlinference.InitExecutionContext",
                    arg: Cow::Borrowed(&buf),
                },
                None,
            )
            .await?;
        let value = deserialize(&resp).map_err(|e| {
            RpcError::Deser(format!("response to {}: {}", "InitExecutionContext", e))
        })?;
        Ok(value)
    }
    #[allow(unused)]
    /// set_input
    async fn set_input(&self, ctx: &Context, arg: &SetInputStruct) -> RpcResult<BaseResult> {
        let buf = serialize(arg)?;
        let resp = self
            .transport
            .send(
                ctx,
                Message {
                    method: "Mlinference.SetInput",
                    arg: Cow::Borrowed(&buf),
                },
                None,
            )
            .await?;
        let value = deserialize(&resp)
            .map_err(|e| RpcError::Deser(format!("response to {}: {}", "SetInput", e)))?;
        Ok(value)
    }
    #[allow(unused)]
    /// compute
    async fn compute(&self, ctx: &Context, arg: &GraphExecutionContext) -> RpcResult<BaseResult> {
        let buf = serialize(arg)?;
        let resp = self
            .transport
            .send(
                ctx,
                Message {
                    method: "Mlinference.Compute",
                    arg: Cow::Borrowed(&buf),
                },
                None,
            )
            .await?;
        let value = deserialize(&resp)
            .map_err(|e| RpcError::Deser(format!("response to {}: {}", "Compute", e)))?;
        Ok(value)
    }
}