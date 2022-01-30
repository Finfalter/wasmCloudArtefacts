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

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct InferenceRequest {
    #[serde(default)]
    pub model: String,
    pub tensor: Tensor,
    pub index: u32,
}

/// InferenceResult
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct InferenceResult {
    pub result: ResultStatus,
    pub tensor: Tensor,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct MlError {
    #[serde(rename = "modelError")]
    pub model_error: u8,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResultStatus {
    #[serde(rename = "hasError")]
    #[serde(default)]
    pub has_error: bool,
    #[serde(rename = "Error")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<MlError>,
}

/// The tensor's dimensions and type are provided as metadata to a model.
/// Any metadata shall be associated to the respective model in a blob store.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Tensor {
    pub data: TensorData,
    pub dimensions: TensorDimensions,
}

pub type TensorData = Vec<u8>;

pub type TensorDimensions = Vec<u32>;

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
    /// predict
    async fn predict(&self, ctx: &Context, arg: &InferenceRequest) -> RpcResult<InferenceResult>;
}

/// MlinferenceReceiver receives messages defined in the Mlinference service trait
/// The Mlinference service
#[doc(hidden)]
#[async_trait]
pub trait MlinferenceReceiver: MessageDispatch + Mlinference {
    async fn dispatch(&self, ctx: &Context, message: &Message<'_>) -> RpcResult<Message<'_>> {
        match message.method {
            "Predict" => {
                let value: InferenceRequest = deserialize(message.arg.as_ref())
                    .map_err(|e| RpcError::Deser(format!("message '{}': {}", message.method, e)))?;
                let resp = Mlinference::predict(self, ctx, &value).await?;
                let buf = serialize(&resp)?;
                Ok(Message {
                    method: "Mlinference.Predict",
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
    /// predict
    async fn predict(&self, ctx: &Context, arg: &InferenceRequest) -> RpcResult<InferenceResult> {
        let buf = serialize(arg)?;
        let resp = self
            .transport
            .send(
                ctx,
                Message {
                    method: "Mlinference.Predict",
                    arg: Cow::Borrowed(&buf),
                },
                None,
            )
            .await?;
        let value = deserialize(&resp)
            .map_err(|e| RpcError::Deser(format!("response to {}: {}", "Predict", e)))?;
        Ok(value)
    }
}
