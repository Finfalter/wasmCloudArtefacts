// This file is generated automatically using wasmcloud/weld-codegen 0.4.3

#[allow(unused_imports)]
use async_trait::async_trait;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use std::{borrow::Borrow, borrow::Cow, io::Write, string::ToString};
#[allow(unused_imports)]
use wasmbus_rpc::{
    cbor::*,
    common::{
        deserialize, message_format, serialize, Context, Message, MessageDispatch, MessageFormat,
        SendOpts, Transport,
    },
    error::{RpcError, RpcResult},
    Timestamp,
};

#[allow(dead_code)]
pub const SMITHY_VERSION: &str = "1.0";

/// Description of Mlpreprocessing service
/// wasmbus.actorReceive
#[async_trait]
pub trait MlPreprocessing {
    /// Converts the input string to a result
    async fn convert<TS: ToString + ?Sized + std::marker::Sync>(
        &self,
        ctx: &Context,
        arg: &TS,
    ) -> RpcResult<String>;
}

/// MlPreprocessingReceiver receives messages defined in the MlPreprocessing service trait
/// Description of Mlpreprocessing service
#[doc(hidden)]
#[async_trait]
pub trait MlPreprocessingReceiver: MessageDispatch + MlPreprocessing {
    async fn dispatch<'disp__, 'ctx__, 'msg__>(
        &'disp__ self,
        ctx: &'ctx__ Context,
        message: &Message<'msg__>,
    ) -> Result<Message<'msg__>, RpcError> {
        match message.method {
            "Convert" => {
                let value: String = wasmbus_rpc::common::deserialize(&message.arg)
                    .map_err(|e| RpcError::Deser(format!("'String': {}", e)))?;
                let resp = MlPreprocessing::convert(self, ctx, &value).await?;
                let buf = wasmbus_rpc::common::serialize(&resp)?;
                Ok(Message {
                    method: "MlPreprocessing.Convert",
                    arg: Cow::Owned(buf),
                })
            }
            _ => Err(RpcError::MethodNotHandled(format!(
                "MlPreprocessing::{}",
                message.method
            ))),
        }
    }
}

/// MlPreprocessingSender sends messages to a MlPreprocessing service
/// Description of Mlpreprocessing service
/// client for sending MlPreprocessing messages
#[derive(Debug)]
pub struct MlPreprocessingSender<T: Transport> {
    transport: T,
}

impl<T: Transport> MlPreprocessingSender<T> {
    /// Constructs a MlPreprocessingSender with the specified transport
    pub fn via(transport: T) -> Self {
        Self { transport }
    }

    pub fn set_timeout(&self, interval: std::time::Duration) {
        self.transport.set_timeout(interval);
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<'send> MlPreprocessingSender<wasmbus_rpc::provider::ProviderTransport<'send>> {
    /// Constructs a Sender using an actor's LinkDefinition,
    /// Uses the provider's HostBridge for rpc
    pub fn for_actor(ld: &'send wasmbus_rpc::core::LinkDefinition) -> Self {
        Self {
            transport: wasmbus_rpc::provider::ProviderTransport::new(ld, None),
        }
    }
}
#[cfg(target_arch = "wasm32")]
impl MlPreprocessingSender<wasmbus_rpc::actor::prelude::WasmHost> {
    /// Constructs a client for actor-to-actor messaging
    /// using the recipient actor's public key
    pub fn to_actor(actor_id: &str) -> Self {
        let transport =
            wasmbus_rpc::actor::prelude::WasmHost::to_actor(actor_id.to_string()).unwrap();
        Self { transport }
    }
}
#[async_trait]
impl<T: Transport + std::marker::Sync + std::marker::Send> MlPreprocessing
    for MlPreprocessingSender<T>
{
    #[allow(unused)]
    /// Converts the input string to a result
    async fn convert<TS: ToString + ?Sized + std::marker::Sync>(
        &self,
        ctx: &Context,
        arg: &TS,
    ) -> RpcResult<String> {
        let buf = wasmbus_rpc::common::serialize(&arg.to_string())?;
        let resp = self
            .transport
            .send(
                ctx,
                Message {
                    method: "MlPreprocessing.Convert",
                    arg: Cow::Borrowed(&buf),
                },
                None,
            )
            .await?;

        let value: String = wasmbus_rpc::common::deserialize(&resp)
            .map_err(|e| RpcError::Deser(format!("'{}': String", e)))?;
        Ok(value)
    }
}
