
use agave_geyser_plugin_interface::geyser_plugin_interface::GeyserPlugin;

mod config;
mod event;
mod filter;
mod plugin;
mod prom;
mod publisher;

pub use {
    config::{Config, ConfigFilter, RedisConfig},
    event::*,
    filter::Filter,
    plugin::RedisPlugin,
    prom::PrometheusService,
    publisher::Publisher,
};

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub unsafe extern "C" fn _create_plugin() -> *mut dyn GeyserPlugin {
    let plugin = RedisPlugin::new();
    let plugin: Box<dyn GeyserPlugin> = Box::new(plugin);
    Box::into_raw(plugin)
}
