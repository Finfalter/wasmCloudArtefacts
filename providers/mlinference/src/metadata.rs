use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelDefinition {
    /// Model name (optional)
    #[serde(default)]
    pub model_name: Option<String>,
    
    /// graph encoding
    #[serde(default)]
    pub graph_encoding: String,
    
    /// execution target
    #[serde(default)]
    pub execution_target: String,
    
    /// tensor type
    #[serde(default)]
    pub tensor_type: String,
    
    /// tensor dimensions in (optional)
    #[serde(default)]
    pub tensor_dimensions_in: Option<Vec<u32>>,
    
    /// tensor dimensions out (optional)
    #[serde(default)]
    pub tensor_dimensions_out: Option<Vec<u32>>
}

/// get first member of
pub fn get_first_member_of(parcels: &Vec<bindle::Parcel>, group: &str) -> Result<bindle::Parcel,()> {
    let members = parcels
    .into_iter()            
    .filter(|parcel| 
        parcel.conditions.is_some() && 
        parcel.conditions.as_ref().unwrap().member_of.is_some()
    )
    .filter(|parcel| parcel.conditions.clone().unwrap().member_of.unwrap().iter().any(|mbs| *mbs == group) )
    .collect::<Vec<&bindle::Parcel>>();

    return match members.len() {
        0 => Err(()),
        _ => Ok(members.first().unwrap().clone().clone())
    };
}