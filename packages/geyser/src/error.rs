use thiserror::Error;

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("IDL error: {0}")]
    IdlError(String),
    #[error("Component not initialized: {0}")]
    NotInitialized(String),
    #[error("Failed to send message to worker channel: {0}")]
    ChannelSendError(String)
}