
use agave_geyser_plugin_interface::geyser_plugin_interface::SlotStatus as PluginSlotStatus;

include!(concat!(
    env!("OUT_DIR"),
    "/heimdall.types.rs"
));

impl From<PluginSlotStatus> for SlotStatus {
    fn from(other: PluginSlotStatus) -> Self {
        match other {
            PluginSlotStatus::Processed => SlotStatus::Processed,
            PluginSlotStatus::Rooted => SlotStatus::Rooted,
            PluginSlotStatus::Confirmed => SlotStatus::Confirmed,
            PluginSlotStatus::FirstShredReceived => SlotStatus::FirstShredReceived,
            PluginSlotStatus::Completed => SlotStatus::Completed,
            PluginSlotStatus::CreatedBank => SlotStatus::CreatedBank,
            PluginSlotStatus::Dead(_) => SlotStatus::Dead,
        }
    }
}
