// This file is generated automatically using wasmcloud/weld-codegen 0.3.2

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

pub const SMITHY_VERSION: &str = "1.0";

/// ComputeOutput
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ComputeOutput {
    pub result: ResultStatus,
    pub tensor: Tensor,
}

// Encode ComputeOutput as CBOR and append to output stream
#[doc(hidden)]
pub fn encode_compute_output<W: wasmbus_rpc::cbor::Write>(
    e: &mut wasmbus_rpc::cbor::Encoder<W>,
    val: &ComputeOutput,
) -> RpcResult<()> {
    e.array(2)?;
    encode_result_status(e, &val.result)?;
    encode_tensor(e, &val.tensor)?;
    Ok(())
}

// Decode ComputeOutput from cbor input stream
#[doc(hidden)]
pub fn decode_compute_output(
    d: &mut wasmbus_rpc::cbor::Decoder<'_>,
) -> Result<ComputeOutput, RpcError> {
    let __result = {
        let mut result: Option<ResultStatus> = None;
        let mut tensor: Option<Tensor> = None;

        let is_array = match d.datatype()? {
            wasmbus_rpc::cbor::Type::Array => true,
            wasmbus_rpc::cbor::Type::Map => false,
            _ => {
                return Err(RpcError::Deser(
                    "decoding struct ComputeOutput, expected array or map".to_string(),
                ))
            }
        };
        if is_array {
            let len = d.array()?.ok_or_else(|| {
                RpcError::Deser(
                    "decoding struct ComputeOutput: indefinite array not supported".to_string(),
                )
            })?;
            for __i in 0..(len as usize) {
                match __i {
                    0 => {
                        result = Some(
                            decode_result_status(d)
                                .map_err(|e| format!("decoding 'ResultStatus': {}", e))?,
                        )
                    }
                    1 => {
                        tensor = Some(
                            decode_tensor(d).map_err(|e| format!("decoding 'Tensor': {}", e))?,
                        )
                    }
                    _ => d.skip()?,
                }
            }
        } else {
            let len = d.map()?.ok_or_else(|| {
                RpcError::Deser(
                    "decoding struct ComputeOutput: indefinite map not supported".to_string(),
                )
            })?;
            for __i in 0..(len as usize) {
                match d.str()? {
                    "result" => {
                        result = Some(
                            decode_result_status(d)
                                .map_err(|e| format!("decoding 'ResultStatus': {}", e))?,
                        )
                    }
                    "tensor" => {
                        tensor = Some(
                            decode_tensor(d).map_err(|e| format!("decoding 'Tensor': {}", e))?,
                        )
                    }
                    _ => d.skip()?,
                }
            }
        }
        ComputeOutput {
            result: if let Some(__x) = result {
                __x
            } else {
                return Err(RpcError::Deser(
                    "missing field ComputeOutput.result (#0)".to_string(),
                ));
            },

            tensor: if let Some(__x) = tensor {
                __x
            } else {
                return Err(RpcError::Deser(
                    "missing field ComputeOutput.tensor (#1)".to_string(),
                ));
            },
        }
    };
    Ok(__result)
}
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ComputeRequest {
    #[serde(default)]
    pub model: String,
    pub tensor: Tensor,
    #[serde(default)]
    pub index: u32,
}

// Encode ComputeRequest as CBOR and append to output stream
#[doc(hidden)]
pub fn encode_compute_request<W: wasmbus_rpc::cbor::Write>(
    e: &mut wasmbus_rpc::cbor::Encoder<W>,
    val: &ComputeRequest,
) -> RpcResult<()> {
    e.array(3)?;
    e.str(&val.model)?;
    encode_tensor(e, &val.tensor)?;
    e.u32(val.index)?;
    Ok(())
}

// Decode ComputeRequest from cbor input stream
#[doc(hidden)]
pub fn decode_compute_request(
    d: &mut wasmbus_rpc::cbor::Decoder<'_>,
) -> Result<ComputeRequest, RpcError> {
    let __result = {
        let mut model: Option<String> = None;
        let mut tensor: Option<Tensor> = None;
        let mut index: Option<u32> = None;

        let is_array = match d.datatype()? {
            wasmbus_rpc::cbor::Type::Array => true,
            wasmbus_rpc::cbor::Type::Map => false,
            _ => {
                return Err(RpcError::Deser(
                    "decoding struct ComputeRequest, expected array or map".to_string(),
                ))
            }
        };
        if is_array {
            let len = d.array()?.ok_or_else(|| {
                RpcError::Deser(
                    "decoding struct ComputeRequest: indefinite array not supported".to_string(),
                )
            })?;
            for __i in 0..(len as usize) {
                match __i {
                    0 => model = Some(d.str()?.to_string()),
                    1 => {
                        tensor = Some(
                            decode_tensor(d).map_err(|e| format!("decoding 'Tensor': {}", e))?,
                        )
                    }
                    2 => index = Some(d.u32()?),
                    _ => d.skip()?,
                }
            }
        } else {
            let len = d.map()?.ok_or_else(|| {
                RpcError::Deser(
                    "decoding struct ComputeRequest: indefinite map not supported".to_string(),
                )
            })?;
            for __i in 0..(len as usize) {
                match d.str()? {
                    "model" => model = Some(d.str()?.to_string()),
                    "tensor" => {
                        tensor = Some(
                            decode_tensor(d).map_err(|e| format!("decoding 'Tensor': {}", e))?,
                        )
                    }
                    "index" => index = Some(d.u32()?),
                    _ => d.skip()?,
                }
            }
        }
        ComputeRequest {
            model: if let Some(__x) = model {
                __x
            } else {
                return Err(RpcError::Deser(
                    "missing field ComputeRequest.model (#0)".to_string(),
                ));
            },

            tensor: if let Some(__x) = tensor {
                __x
            } else {
                return Err(RpcError::Deser(
                    "missing field ComputeRequest.tensor (#1)".to_string(),
                ));
            },

            index: if let Some(__x) = index {
                __x
            } else {
                return Err(RpcError::Deser(
                    "missing field ComputeRequest.index (#2)".to_string(),
                ));
            },
        }
    };
    Ok(__result)
}
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct MlError {
    #[serde(default)]
    pub err: u8,
}

// Encode MlError as CBOR and append to output stream
#[doc(hidden)]
pub fn encode_ml_error<W: wasmbus_rpc::cbor::Write>(
    e: &mut wasmbus_rpc::cbor::Encoder<W>,
    val: &MlError,
) -> RpcResult<()> {
    e.map(1)?;
    e.str("err")?;
    e.u8(val.err)?;
    Ok(())
}

// Decode MlError from cbor input stream
#[doc(hidden)]
pub fn decode_ml_error(d: &mut wasmbus_rpc::cbor::Decoder<'_>) -> Result<MlError, RpcError> {
    let __result = {
        let mut err: Option<u8> = None;

        let is_array = match d.datatype()? {
            wasmbus_rpc::cbor::Type::Array => true,
            wasmbus_rpc::cbor::Type::Map => false,
            _ => {
                return Err(RpcError::Deser(
                    "decoding struct MlError, expected array or map".to_string(),
                ))
            }
        };
        if is_array {
            let len = d.array()?.ok_or_else(|| {
                RpcError::Deser(
                    "decoding struct MlError: indefinite array not supported".to_string(),
                )
            })?;
            for __i in 0..(len as usize) {
                match __i {
                    0 => err = Some(d.u8()?),
                    _ => d.skip()?,
                }
            }
        } else {
            let len = d.map()?.ok_or_else(|| {
                RpcError::Deser("decoding struct MlError: indefinite map not supported".to_string())
            })?;
            for __i in 0..(len as usize) {
                match d.str()? {
                    "err" => err = Some(d.u8()?),
                    _ => d.skip()?,
                }
            }
        }
        MlError {
            err: if let Some(__x) = err {
                __x
            } else {
                return Err(RpcError::Deser(
                    "missing field MlError.err (#0)".to_string(),
                ));
            },
        }
    };
    Ok(__result)
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

// Encode ResultStatus as CBOR and append to output stream
#[doc(hidden)]
pub fn encode_result_status<W: wasmbus_rpc::cbor::Write>(
    e: &mut wasmbus_rpc::cbor::Encoder<W>,
    val: &ResultStatus,
) -> RpcResult<()> {
    e.array(2)?;
    e.bool(val.has_error)?;
    if let Some(val) = val.error.as_ref() {
        encode_ml_error(e, val)?;
    } else {
        e.null()?;
    }
    Ok(())
}

// Decode ResultStatus from cbor input stream
#[doc(hidden)]
pub fn decode_result_status(
    d: &mut wasmbus_rpc::cbor::Decoder<'_>,
) -> Result<ResultStatus, RpcError> {
    let __result = {
        let mut has_error: Option<bool> = None;
        let mut error: Option<Option<MlError>> = Some(None);

        let is_array = match d.datatype()? {
            wasmbus_rpc::cbor::Type::Array => true,
            wasmbus_rpc::cbor::Type::Map => false,
            _ => {
                return Err(RpcError::Deser(
                    "decoding struct ResultStatus, expected array or map".to_string(),
                ))
            }
        };
        if is_array {
            let len = d.array()?.ok_or_else(|| {
                RpcError::Deser(
                    "decoding struct ResultStatus: indefinite array not supported".to_string(),
                )
            })?;
            for __i in 0..(len as usize) {
                match __i {
                    0 => has_error = Some(d.bool()?),
                    1 => {
                        error = if wasmbus_rpc::cbor::Type::Null == d.datatype()? {
                            d.skip()?;
                            Some(None)
                        } else {
                            Some(Some(
                                decode_ml_error(d)
                                    .map_err(|e| format!("decoding 'MlError': {}", e))?,
                            ))
                        }
                    }

                    _ => d.skip()?,
                }
            }
        } else {
            let len = d.map()?.ok_or_else(|| {
                RpcError::Deser(
                    "decoding struct ResultStatus: indefinite map not supported".to_string(),
                )
            })?;
            for __i in 0..(len as usize) {
                match d.str()? {
                    "hasError" => has_error = Some(d.bool()?),
                    "Error" => {
                        error = if wasmbus_rpc::cbor::Type::Null == d.datatype()? {
                            d.skip()?;
                            Some(None)
                        } else {
                            Some(Some(
                                decode_ml_error(d)
                                    .map_err(|e| format!("decoding 'MlError': {}", e))?,
                            ))
                        }
                    }
                    _ => d.skip()?,
                }
            }
        }
        ResultStatus {
            has_error: if let Some(__x) = has_error {
                __x
            } else {
                return Err(RpcError::Deser(
                    "missing field ResultStatus.has_error (#0)".to_string(),
                ));
            },
            error: error.unwrap(),
        }
    };
    Ok(__result)
}
/// The tensor's dimensions and type are provided as metadata to a model.
/// Any metadata shall be associated to the respective model in a blob store.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Tensor {
    pub ttype: TensorType,
    pub dimensions: TensorDimensions,
    #[serde(with = "serde_bytes")]
    #[serde(default)]
    pub data: Vec<u8>,
}

// Encode Tensor as CBOR and append to output stream
#[doc(hidden)]
pub fn encode_tensor<W: wasmbus_rpc::cbor::Write>(
    e: &mut wasmbus_rpc::cbor::Encoder<W>,
    val: &Tensor,
) -> RpcResult<()> {
    e.array(3)?;
    encode_tensor_type(e, &val.ttype)?;
    encode_tensor_dimensions(e, &val.dimensions)?;
    e.bytes(&val.data)?;
    Ok(())
}

// Decode Tensor from cbor input stream
#[doc(hidden)]
pub fn decode_tensor(d: &mut wasmbus_rpc::cbor::Decoder<'_>) -> Result<Tensor, RpcError> {
    let __result = {
        let mut ttype: Option<TensorType> = None;
        let mut dimensions: Option<TensorDimensions> = None;
        let mut data: Option<Vec<u8>> = None;

        let is_array = match d.datatype()? {
            wasmbus_rpc::cbor::Type::Array => true,
            wasmbus_rpc::cbor::Type::Map => false,
            _ => {
                return Err(RpcError::Deser(
                    "decoding struct Tensor, expected array or map".to_string(),
                ))
            }
        };
        if is_array {
            let len = d.array()?.ok_or_else(|| {
                RpcError::Deser(
                    "decoding struct Tensor: indefinite array not supported".to_string(),
                )
            })?;
            for __i in 0..(len as usize) {
                match __i {
                    0 => {
                        ttype = Some(
                            decode_tensor_type(d)
                                .map_err(|e| format!("decoding 'TensorType': {}", e))?,
                        )
                    }
                    1 => {
                        dimensions = Some(
                            decode_tensor_dimensions(d)
                                .map_err(|e| format!("decoding 'TensorDimensions': {}", e))?,
                        )
                    }
                    2 => data = Some(d.bytes()?.to_vec()),
                    _ => d.skip()?,
                }
            }
        } else {
            let len = d.map()?.ok_or_else(|| {
                RpcError::Deser("decoding struct Tensor: indefinite map not supported".to_string())
            })?;
            for __i in 0..(len as usize) {
                match d.str()? {
                    "ttype" => {
                        ttype = Some(
                            decode_tensor_type(d)
                                .map_err(|e| format!("decoding 'TensorType': {}", e))?,
                        )
                    }
                    "dimensions" => {
                        dimensions = Some(
                            decode_tensor_dimensions(d)
                                .map_err(|e| format!("decoding 'TensorDimensions': {}", e))?,
                        )
                    }
                    "data" => data = Some(d.bytes()?.to_vec()),
                    _ => d.skip()?,
                }
            }
        }
        Tensor {
            ttype: if let Some(__x) = ttype {
                __x
            } else {
                return Err(RpcError::Deser(
                    "missing field Tensor.ttype (#0)".to_string(),
                ));
            },

            dimensions: if let Some(__x) = dimensions {
                __x
            } else {
                return Err(RpcError::Deser(
                    "missing field Tensor.dimensions (#1)".to_string(),
                ));
            },

            data: if let Some(__x) = data {
                __x
            } else {
                return Err(RpcError::Deser(
                    "missing field Tensor.data (#2)".to_string(),
                ));
            },
        }
    };
    Ok(__result)
}
pub type TensorDimensions = Vec<u32>;

// Encode TensorDimensions as CBOR and append to output stream
#[doc(hidden)]
pub fn encode_tensor_dimensions<W: wasmbus_rpc::cbor::Write>(
    e: &mut wasmbus_rpc::cbor::Encoder<W>,
    val: &TensorDimensions,
) -> RpcResult<()> {
    e.array(val.len() as u64)?;
    for item in val.iter() {
        e.u32(*item)?;
    }
    Ok(())
}

// Decode TensorDimensions from cbor input stream
#[doc(hidden)]
pub fn decode_tensor_dimensions(
    d: &mut wasmbus_rpc::cbor::Decoder<'_>,
) -> Result<TensorDimensions, RpcError> {
    let __result = {
        if let Some(n) = d.array()? {
            let mut arr: Vec<u32> = Vec::with_capacity(n as usize);
            for _ in 0..(n as usize) {
                arr.push(d.u32()?)
            }
            arr
        } else {
            // indefinite array
            let mut arr: Vec<u32> = Vec::new();
            loop {
                match d.datatype() {
                    Err(_) => break,
                    Ok(wasmbus_rpc::cbor::Type::Break) => break,
                    Ok(_) => arr.push(d.u32()?),
                }
            }
            arr
        }
    };
    Ok(__result)
}
/// TensorType
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct TensorType {
    #[serde(default)]
    pub ttype: u8,
}

// Encode TensorType as CBOR and append to output stream
#[doc(hidden)]
pub fn encode_tensor_type<W: wasmbus_rpc::cbor::Write>(
    e: &mut wasmbus_rpc::cbor::Encoder<W>,
    val: &TensorType,
) -> RpcResult<()> {
    e.map(1)?;
    e.str("ttype")?;
    e.u8(val.ttype)?;
    Ok(())
}

// Decode TensorType from cbor input stream
#[doc(hidden)]
pub fn decode_tensor_type(d: &mut wasmbus_rpc::cbor::Decoder<'_>) -> Result<TensorType, RpcError> {
    let __result = {
        let mut ttype: Option<u8> = None;

        let is_array = match d.datatype()? {
            wasmbus_rpc::cbor::Type::Array => true,
            wasmbus_rpc::cbor::Type::Map => false,
            _ => {
                return Err(RpcError::Deser(
                    "decoding struct TensorType, expected array or map".to_string(),
                ))
            }
        };
        if is_array {
            let len = d.array()?.ok_or_else(|| {
                RpcError::Deser(
                    "decoding struct TensorType: indefinite array not supported".to_string(),
                )
            })?;
            for __i in 0..(len as usize) {
                match __i {
                    0 => ttype = Some(d.u8()?),
                    _ => d.skip()?,
                }
            }
        } else {
            let len = d.map()?.ok_or_else(|| {
                RpcError::Deser(
                    "decoding struct TensorType: indefinite map not supported".to_string(),
                )
            })?;
            for __i in 0..(len as usize) {
                match d.str()? {
                    "ttype" => ttype = Some(d.u8()?),
                    _ => d.skip()?,
                }
            }
        }
        TensorType {
            ttype: if let Some(__x) = ttype {
                __x
            } else {
                return Err(RpcError::Deser(
                    "missing field TensorType.ttype (#0)".to_string(),
                ));
            },
        }
    };
    Ok(__result)
}
/// Description of Converter service
/// wasmbus.actorReceive
#[async_trait]
pub trait Converter {
    /// Compute
    async fn compute(&self, ctx: &Context, arg: &ComputeRequest) -> RpcResult<ComputeOutput>;
}

/// ConverterReceiver receives messages defined in the Converter service trait
/// Description of Converter service
#[doc(hidden)]
#[async_trait]
pub trait ConverterReceiver: MessageDispatch + Converter {
    async fn dispatch(&self, ctx: &Context, message: &Message<'_>) -> RpcResult<Message<'_>> {
        match message.method {
            "Compute" => {
                let value: ComputeRequest =
                    wasmbus_rpc::common::decode(&message.arg, &decode_compute_request)
                        .map_err(|e| RpcError::Deser(format!("'ComputeRequest': {}", e)))?;
                let resp = Converter::compute(self, ctx, &value).await?;
                let mut e = wasmbus_rpc::cbor::vec_encoder();
                encode_compute_output(&mut e, &resp)?;
                let buf = e.into_inner();
                Ok(Message {
                    method: "Converter.Compute",
                    arg: Cow::Owned(buf),
                })
            }
            _ => Err(RpcError::MethodNotHandled(format!(
                "Converter::{}",
                message.method
            ))),
        }
    }
}

/// ConverterSender sends messages to a Converter service
/// Description of Converter service
/// client for sending Converter messages
#[derive(Debug)]
pub struct ConverterSender<T: Transport> {
    transport: T,
}

impl<T: Transport> ConverterSender<T> {
    /// Constructs a ConverterSender with the specified transport
    pub fn via(transport: T) -> Self {
        Self { transport }
    }

    pub fn set_timeout(&self, interval: std::time::Duration) {
        self.transport.set_timeout(interval);
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<'send> ConverterSender<wasmbus_rpc::provider::ProviderTransport<'send>> {
    /// Constructs a Sender using an actor's LinkDefinition,
    /// Uses the provider's HostBridge for rpc
    pub fn for_actor(ld: &'send wasmbus_rpc::core::LinkDefinition) -> Self {
        Self {
            transport: wasmbus_rpc::provider::ProviderTransport::new(ld, None),
        }
    }
}
#[cfg(target_arch = "wasm32")]
impl ConverterSender<wasmbus_rpc::actor::prelude::WasmHost> {
    /// Constructs a client for actor-to-actor messaging
    /// using the recipient actor's public key
    pub fn to_actor(actor_id: &str) -> Self {
        let transport =
            wasmbus_rpc::actor::prelude::WasmHost::to_actor(actor_id.to_string()).unwrap();
        Self { transport }
    }
}
#[async_trait]
impl<T: Transport + std::marker::Sync + std::marker::Send> Converter for ConverterSender<T> {
    #[allow(unused)]
    /// Compute
    async fn compute(&self, ctx: &Context, arg: &ComputeRequest) -> RpcResult<ComputeOutput> {
        let mut e = wasmbus_rpc::cbor::vec_encoder();
        encode_compute_request(&mut e, arg)?;
        let buf = e.into_inner();
        let resp = self
            .transport
            .send(
                ctx,
                Message {
                    method: "Converter.Compute",
                    arg: Cow::Borrowed(&buf),
                },
                None,
            )
            .await?;

        let value: ComputeOutput = wasmbus_rpc::common::decode(&resp, &decode_compute_output)
            .map_err(|e| RpcError::Deser(format!("'{}': ComputeOutput", e)))?;
        Ok(value)
    }
}
