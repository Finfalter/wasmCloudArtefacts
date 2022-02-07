//use hashmap_ci::{make_case_insensitive};
use std::{str::FromStr};
use std::{collections::HashMap};
use wasmbus_rpc::{RpcError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelSettings {
    /// model to path assignments
    #[serde(default)]
    pub models: Models,

    /// loading models before first compute or at linkage
    pub lazy_load: Option<bool>
}

impl Default for ModelSettings {
    fn default() -> ModelSettings {
        ModelSettings {
            models: Models::default(),
            lazy_load: Some(false),
        }
    }
}

impl ModelSettings {
    /// perform additional validation checks on settings.
    /// Several checks have already been done during deserialization.
    /// All errors found are combined into a single error message
    fn validate(&self) -> Result<(), RpcError> {
        Ok(())
    }
}

pub fn load_settings(values: &HashMap<String, String>) -> Result<ModelSettings, RpcError> {
    // Allow keys to be UPPERCASE, as an accommodation
    // for the lost souls who prefer ugly all-caps variable names.
    let values = crate::make_case_insensitive(values).ok_or_else(|| RpcError::InvalidParameter(
        "Key collision: httpserver settings (from linkdef.values) has one or more keys that are not unique based on case-insensitivity"
            .to_string(),
    ))?;

    let mut settings = ModelSettings::default();

    if let Some(cj) = values.get("config_b64") {
        settings = serde_json::from_slice(
            &base64::decode(cj)
                .map_err(|_| RpcError::ProviderInit("invalid config_base64 encoding".into()))?,
        )
        .map_err(|e| RpcError::ProviderInit(format!("invalid json config: {}", e)))?;
    }
    if let Some(cj) = values.get("config_json") {
        settings = serde_json::from_str(cj.as_str())
            .map_err(|e| RpcError::ProviderInit(format!("invalid json config: {}", e)))?;
    }
   
    if let Some(lazy_load) = values.get("lazy_load") {
        settings.lazy_load = FromStr::from_str(lazy_load).ok();
    }
   
    if settings.models.is_empty() {
        Err(RpcError::ProviderInit(
            "link params values are missing 'uri'".into(),
        ))
    } else {
        settings.validate()?;
        Ok(settings)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Models {
    pub zoo: HashMap<crate::ModelName, crate::BindlePath>
}

impl Models {
    fn is_empty(&mut self) -> bool {
        self.zoo.is_empty()
    }
}

