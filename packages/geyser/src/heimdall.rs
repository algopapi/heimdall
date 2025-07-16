use agave_geyser_plugin_interface::geyser_plugin_interface::GeyserPlugin;

use crate::{parser::GenericParser, publisher::RedisPublisher};


pub struct Heimdall {
    parser: Option<GenericParser>,
    publisher: Option<RedisPublisher>
}

impl Default for Heimdall {
    fn default() -> Self {
        Self { parser: None, publisher: None }
    }
}

impl std::fmt::Debug for Heimdall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Heimdall").finish()
    }
}

impl GeyserPlugin for Heimdall {
    fn name(&self) -> &'static str {
        "Heimdall"
    }
}