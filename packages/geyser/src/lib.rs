use agave_geyser_plugin_interface::geyser_plugin_interface::GeyserPlugin;

use crate::heimdall::Heimdall;

mod config;
mod error;
mod heimdall;
mod idl;
mod model;
mod parser;
mod publisher;

#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
pub unsafe extern "C" fn _create_plugin() -> *mut dyn GeyserPlugin {
    let plugin = Heimdall::default();
    let plugin: Box<dyn GeyserPlugin> = Box::new(plugin);
    Box::into_raw(plugin)
}