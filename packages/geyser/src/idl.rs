use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

use serde::Deserialize;

use crate::error::PluginError;

#[derive(Debug, Deserialize)]
pub struct Idl {
    pub address: String,
    #[serde(default)]
    pub accounts: Vec<IdlAccount>,
    #[serde(default)]
    pub types: Vec<IdlTypeDef>,
    #[serde(default)]
    pub events: Vec<IdlEvent>
}

#[derive(Debug, Deserialize)]
pub struct IdlAccount {
    pub name: String,
    pub discriminator: Vec<u8>
}

#[derive(Debug, Deserialize)]
pub struct IdlEvent {
    pub name: String,
    pub discriminator: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub struct IdlTypeDef {
    pub name: String,
    #[serde(rename = "type")]
    pub type_def: IdlTypeDefStruct
}

#[derive(Debug, Deserialize)]
pub struct IdlTypeDefStruct {
    pub kind: String,
    pub fields: Vec<IdlField>
}

#[derive(Debug, Deserialize)]
pub struct IdlField {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: IdlType
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum IdlType {
    Simple(String),
    Complex(HashMap<String, IdlType>)
}

impl Idl {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, PluginError> {
        let file = File::open(path).map_err(|e| PluginError::IdlError(e.to_string()))?;
        let reader = BufReader::new(file);
        let idl: Idl = serde_json::from_reader(reader)
            .map_err(|e| PluginError::IdlError(e.to_string()))?;

        Ok(idl)
    }
}


