use crate::Error;
use std::path::PathBuf;
use std::{collections::HashMap};
//use hashmap_ci::{make_case_insensitive};
use serde::{de::Deserializer, de::Visitor, Deserialize, Serialize};

macro_rules! merge {
    ( $self:ident, $other: ident, $( $field:ident),+ ) => {
        $(
            if $other.$field.is_some() {
                $self.$field = $other.$field;
            }
        )*
    };
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelSettings {
    #[serde(default)]
    pub models: Models,
}

impl Default for ModelSettings {
    fn default() -> ModelSettings {
        ModelSettings {
            models: Models::default(),
        }
    }
}

impl ModelSettings {
    /// load Settings from a file with .toml or .json extension
    fn from_file<P: Into<PathBuf>>(fpath: P) -> Result<Self, Error> {
        let fpath: PathBuf = fpath.into();
        let data = std::fs::read(&fpath)
            .map_err(|e| Error::Settings(format!("reading file {}: {}", &fpath.display(), e)))?;
        if let Some(ext) = fpath.extension() {
            let ext = ext.to_string_lossy();
            match ext.as_ref() {
                "json" => ModelSettings::from_json(&data),
                "toml" => ModelSettings::from_toml(&data),
                _ => Err(Error::Settings(format!("unrecognized extension {}", ext))),
            }
        } else {
            Err(Error::Settings(format!(
                "unrecognized file type {}",
                &fpath.display()
            )))
        }
    }

    /// load settings from json
    fn from_json(data: &[u8]) -> Result<Self, Error> {
        serde_json::from_slice(data).map_err(|e| Error::Settings(format!("invalid json: {}", e)))
    }

    /// load settings from toml file
    fn from_toml(data: &[u8]) -> Result<Self, Error> {
        toml::from_slice(data).map_err(Error::SettingsToml)
    }

    /// Merge settings from other into self
    fn merge(&mut self, other: ModelSettings) {
        //merge!(self, other, address);
        //self.models.merge(other.models);
    }

    /// perform additional validation checks on settings.
    /// Several checks have already been done during deserialization.
    /// All errors found are combined into a single error message
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

pub fn load_settings(values: &HashMap<String, String>) -> Result<ModelSettings, Error> {
    // Allow keys to be UPPERCASE, as an accommodation
    // for the lost souls who prefer ugly all-caps variable names.
    let values = crate::make_case_insensitive(values).ok_or_else(|| Error::InvalidParameter(
        "Key collision: httpserver settings (from linkdef.values) has one or more keys that are not unique based on case-insensitivity"
            .to_string(),
    ))?;

    let mut settings = ModelSettings::default();

    if let Some(fpath) = values.get("config_file") {
        settings.merge(ModelSettings::from_file(fpath)?);
    }

    if let Some(str) = values.get("config_b64") {
        let bytes = base64::decode(str.as_bytes())
            .map_err(|e| Error::Settings(format!("invalid base64 encoding: {}", e)))?;
        settings.merge(ModelSettings::from_json(&bytes)?);
    }

    if let Some(str) = values.get("config_json") {
        settings.merge(ModelSettings::from_json(str.as_bytes())?);
    }

    // // accept address as value parameter
    // if let Some(addr) = values.get("address") {
    //     settings.address = Some(
    //         SocketAddr::from_str(addr)
    //             .map_err(|_| Error::InvalidParameter(format!("invalid address: {}", addr)))?,
    //     );
    // }

    settings.validate()?;
    Ok(settings)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Models {
    pub models: HashMap<crate::ModelName, crate::BindlePath>
}

// impl Models {
//     fn merge(&mut self, other: Models) {
//         merge!(self, other, cert_file, priv_key_file);
//     }
// }

// impl Models {
//     pub fn is_set(&self) -> bool {
//         self.cert_file.is_some() && self.priv_key_file.is_some()
//     }
//}