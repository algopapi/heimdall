use std::{fs::File, io::BufReader, path::Path};

use serde::Deserialize;

use crate::error::PluginError;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub redis_url: String,
    pub programs: Vec<ProgramConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ProgramConfig {
    pub program_id: String,
    pub idl_path: String,
    pub account_stream: String,
    #[serde(default)]
    pub events: Vec<EventConfig>
}

#[derive(Debug, Deserialize)]
pub struct EventConfig {
    pub name: String,
    pub stream: String
}

impl Config {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, PluginError> {
        let file = File::open(path).map_err(|e| PluginError::ConfigError(e.to_string()))?;
        let reader = BufReader::new(file);
        let config: Config = serde_json::from_reader(reader)
            .map_err(|e| PluginError::ConfigError(e.to_string()))?;

        Ok(config)
    }
}