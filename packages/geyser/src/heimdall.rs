use agave_geyser_plugin_interface::geyser_plugin_interface::{GeyserPlugin, GeyserPluginError, ReplicaAccountInfoVersions, ReplicaTransactionInfoVersions, SlotStatus};
use log::info;

use crate::{config::Config, parser::GenericParser, publisher::RedisPublisher};


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

    fn on_load(&mut self, _config_file: &str, _is_reload: bool) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        solana_logger::setup_with_default("info");
        info!("loading plugin from config file: {}", _config_file);

        let config = Config::load_from_file(_config_file).map_err(|e| GeyserPluginError::ConfigFileReadError { msg: e.to_string() })?;

        let parser = GenericParser::new(&config).map_err(|e| GeyserPluginError::Custom(Box::new(e)))?;
        self.parser = Some(parser);

        let publisher = RedisPublisher::new(config.redis_url).map_err(|e| GeyserPluginError::Custom(Box::new(e)))?;
        self.publisher = Some(publisher);

        info!("Plugin loaded successfully. Monitoring {} program.", config.programs.len());

        Ok(())
    }

    fn on_unload(&mut self) {
        info!("inloading plugin")
    }

    fn update_account(
        &self,
        account: ReplicaAccountInfoVersions,
        slot: u64,
        _is_startup: bool,
    ) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        let account_info = match account {
            ReplicaAccountInfoVersions::V0_0_3(a) => a,
            _ => return Ok(()),
        };

        let parser = self.parser.as_ref().unwrap();
        let publisher = self.publisher.as_ref().unwrap();

        if let Some(parsed_data) = parser.parse_account(account_info) {
            publisher
                .publish_account_update(parsed_data, slot)
                .map_err(|e| GeyserPluginError::Custom(Box::new(e)))?;
        }

        Ok(())
    }

    fn notify_transaction(
        &self,
        transaction: ReplicaTransactionInfoVersions,
        slot: u64,
    ) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        let transaction_info = match transaction {
            ReplicaTransactionInfoVersions::V0_0_2(t) => t,
            _ => return Ok(()),
        };

        // We only care about successful transactions for parsing events
        if transaction_info.transaction_status_meta.status.is_err() {
            return Ok(());
        }

        let parser = self.parser.as_ref().unwrap();
        let publisher = self.publisher.as_ref().unwrap();

        let events = parser.parse_transaction_events(transaction_info);
        for event in events {
            publisher
                .publish_event(event, slot)
                .map_err(|e| GeyserPluginError::Custom(Box::new(e)))?;
        }

        Ok(())
    }

    fn update_slot_status(
        &self,
        _slot: u64,
        _parent: Option<u64>,
        _status: SlotStatus,
    ) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        Ok(())
    }

    fn account_data_notifications_enabled(&self) -> bool {
        true
    }

    fn transaction_notifications_enabled(&self) -> bool {
        true
    }
}