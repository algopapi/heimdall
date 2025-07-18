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

    pub redis: RedisConfig,

    #[serde(default)]
    pub shutdown_timeout_ms: u64,

    pub filters: Vec<ConfigFilter>,

    #[serde(default)]
    pub prometheus: Option<SocketAddr>,
}

#[derive(Debug, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout_ms: u64,
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
    pub fn read_from<P: AsRef<Path>>(config_path: P) -> PluginResult<Self> {
        let file = File::open(config_path)?;
        let this: Self = serde_json::from_reader(file)
            .map_err(|e| GeyserPluginError::ConfigFileReadError { msg: e.to_string() })?;
        Ok(this)
    }

    pub fn redis_client(&self) -> Result<redis::Client, redis::RedisError> {
        redis::Client::open(self.redis.url.as_str())
    }

    pub fn create_prometheus(&self) -> IoResult<Option<PrometheusService>> {
        self.prometheus.map(PrometheusService::new).transpose()
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct ConfigFilter {
    pub update_account_stream: String,
    pub slot_status_stream: String,
    pub transaction_stream: String,
    pub program_ignores: Vec<String>,
    pub program_filters: Vec<String>,
    pub account_filters: Vec<String>,
    pub publish_all_accounts: bool,
    pub include_vote_transactions: bool,
    pub include_failed_transactions: bool,
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
