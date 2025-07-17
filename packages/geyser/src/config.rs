use {
    crate::PrometheusService,
    agave_geyser_plugin_interface::geyser_plugin_interface::{
        GeyserPluginError, Result as PluginResult,
    },
    serde::Deserialize,
    std::{fs::File, io::Result as IoResult, net::SocketAddr, path::Path},
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[allow(dead_code)]
    libpath: String,

    /// Redis config.
    pub redis: RedisConfig,

    /// Graceful shutdown timeout.
    #[serde(default)]
    pub shutdown_timeout_ms: u64,

    /// Accounts, transactions filters
    pub filters: Vec<ConfigFilter>,

    /// Prometheus endpoint.
    #[serde(default)]
    pub prometheus: Option<SocketAddr>,
}

#[derive(Debug, Deserialize)]
pub struct RedisConfig {
    /// Redis connection URL (e.g., "redis://localhost:6379")
    pub url: String,
    /// Maximum number of connections in the pool
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    /// Connection timeout in milliseconds
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout_ms: u64,
    /// Redis database number
    #[serde(default)]
    pub database: i64,
}

fn default_max_connections() -> u32 {
    10
}

fn default_connection_timeout() -> u64 {
    5000
}

impl Default for Config {
    fn default() -> Self {
        Self {
            libpath: "".to_owned(),
            redis: RedisConfig {
                url: "redis://localhost:6379".to_owned(),
                max_connections: 10,
                connection_timeout_ms: 5000,
                database: 0,
            },
            shutdown_timeout_ms: 30_000,
            filters: vec![],
            prometheus: None,
        }
    }
}

impl Config {
    /// Read plugin from JSON file.
    pub fn read_from<P: AsRef<Path>>(config_path: P) -> PluginResult<Self> {
        let file = File::open(config_path)?;
        let this: Self = serde_json::from_reader(file)
            .map_err(|e| GeyserPluginError::ConfigFileReadError { msg: e.to_string() })?;
        Ok(this)
    }

    /// Create Redis client from config.
    pub fn redis_client(&self) -> Result<redis::Client, redis::RedisError> {
        redis::Client::open(self.redis.url.as_str())
    }

    pub fn create_prometheus(&self) -> IoResult<Option<PrometheusService>> {
        self.prometheus.map(PrometheusService::new).transpose()
    }
}

/// Plugin config.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct ConfigFilter {
    /// Redis stream to send account updates to.
    pub update_account_stream: String,
    /// Redis stream to send slot status updates to.
    pub slot_status_stream: String,
    /// Redis stream to send transaction to.
    pub transaction_stream: String,
    /// List of programs to ignore.
    pub program_ignores: Vec<String>,
    /// List of programs to include
    pub program_filters: Vec<String>,
    // List of accounts to include
    pub account_filters: Vec<String>,
    /// Publish all accounts on startup.
    pub publish_all_accounts: bool,
    /// Publish vote transactions.
    pub include_vote_transactions: bool,
    /// Publish failed transactions.
    pub include_failed_transactions: bool,
    /// Wrap all event message in a single message type.
    pub wrap_messages: bool,
}

impl Default for ConfigFilter {
    fn default() -> Self {
        Self {
            update_account_stream: "".to_owned(),
            slot_status_stream: "".to_owned(),
            transaction_stream: "".to_owned(),
            program_ignores: Vec::new(),
            program_filters: Vec::new(),
            account_filters: Vec::new(),
            publish_all_accounts: false,
            include_vote_transactions: true,
            include_failed_transactions: true,
            wrap_messages: false,
        }
    }
}
