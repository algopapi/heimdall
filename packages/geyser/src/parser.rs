use {
    crate::{
        config::Config,
        error::PluginError,
        idl::{Idl, IdlEvent, IdlTypeDef},
        model::{ParsedAccount, ParsedEvent},
    },
    agave_geyser_plugin_interface::geyser_plugin_interface::{
        ReplicaAccountInfoV3, ReplicaTransactionInfoV2,
    },
    base64,
    log::warn,
    std::collections::HashMap,
};

struct ProgramInfo {
    idl: Idl,
    account_stream: String,
    account_discriminator_map: HashMap<[u8; 8], (String, IdlTypeDef)>,
    event_discriminator_map: HashMap<[u8; 8], (String, String)>,
}

pub struct GenericParser {
    program_map: HashMap<Vec<u8>, ProgramInfo>,
}

impl GenericParser {
    pub fn new(config: &Config) -> Result<Self, PluginError> {
        let mut program_map = HashMap::new();

        for program_config in &config.programs {
            let program_id_bytes = bs58::decode(&program_config.program_id)
                .into_vec()
                .map_err(|e| PluginError::ConfigError(e.to_string()))?;
            let idl = Idl::load_from_file(&program_config.idl_path)?;

            // Build account discriminator map
            let type_def_map: HashMap<String, IdlTypeDef> = idl
                .types
                .iter()
                .map(|t| (t.name.clone(), t.clone()))
                .collect();
            let mut account_discriminator_map = HashMap::new();
            for acc_def in &idl.accounts {
                if let Some(type_def) = type_def_map.get(&acc_def.name) {
                    let disc: [u8; 8] = acc_def.discriminator[0..8].try_into().unwrap();
                    account_discriminator_map
                        .insert(disc, (acc_def.name.clone(), type_def.clone()));
                }
            }

            // Build event discriminator map
            let mut event_discriminator_map = HashMap::new();
            let idl_event_map: HashMap<String, IdlEvent> = idl
                .events
                .iter()
                .map(|e| (e.name.clone(), e.clone()))
                .collect();
            for event_config in &program_config.events {
                if let Some(idl_event) = idl_event_map.get(&event_config.name) {
                    let disc: [u8; 8] = idl_event.discriminator[0..8].try_into().unwrap();
                    event_discriminator_map.insert(
                        disc,
                        (event_config.name.clone(), event_config.stream.clone()),
                    );
                }
            }

            program_map.insert(
                program_id_bytes,
                ProgramInfo {
                    idl,
                    account_stream: program_config.account_stream.clone(),
                    account_discriminator_map,
                    event_discriminator_map,
                },
            );
        }
        Ok(Self { program_map })
    }

    pub fn parse_account<'a>(
        &self,
        account_info: &'a ReplicaAccountInfoV3<'a>,
    ) -> Option<ParsedAccount> {
        if let Some(program_info) = self.program_map.get(account_info.owner) {
            let data = account_info.data;
            if data.len() < 8 {
                return None;
            }

            let mut discriminator = [0u8; 8];
            discriminator.copy_from_slice(&data[..8]);

            if let Some((account_type, _)) =
                program_info.account_discriminator_map.get(&discriminator)
            {
                match serde_json::from_slice(&data[8..]) {
                    Ok(data_json) => Some(ParsedAccount {
                        account_stream: program_info.account_stream.clone(),
                        account_pubkey: bs58::encode(account_info.pubkey).into_string(),
                        data_json,
                    }),
                    Err(e) => {
                        warn!(
                            "JSON deserialization failed for account {}: {}",
                            bs58::encode(account_info.pubkey).into_string(),
                            e
                        );
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn parse_transaction_events<'a>(
        &self,
        tx_info: &'a ReplicaTransactionInfoV2<'a>,
    ) -> Vec<ParsedEvent> {
        let mut parsed_events = Vec::new();
        let log_messages = match tx_info.transaction_status_meta.log_messages.as_ref() {
            Some(logs) => logs,
            None => return parsed_events,
        };

        for log_message in log_messages {
            const EVENT_PREFIX: &str = "Program-log: ";
            if !log_message.starts_with(EVENT_PREFIX) {
                continue;
            }

            let event_data_b64 = &log_message[EVENT_PREFIX.len()..];
            let event_data_bytes = match base64::decode(event_data_b64) {
                Ok(bytes) => bytes,
                Err(_) => continue,
            };

            if event_data_bytes.len() < 8 {
                continue;
            }

            let mut discriminator = [0u8; 8];
            discriminator.copy_from_slice(&event_data_bytes[..8]);

            for (program_id, program_info) in &self.program_map {
                if tx_info
                    .transaction
                    .message()
                    .account_keys()
                    .iter()
                    .any(|key| key.as_ref() == program_id.as_slice())
                {
                    if let Some((event_name, stream)) =
                        program_info.event_discriminator_map.get(&discriminator)
                    {
                        match serde_json::from_slice(&event_data_bytes[8..]) {
                            Ok(data_json) => {
                                let signers: Vec<String> = tx_info
                                    .transaction
                                    .message()
                                    .account_keys()
                                    .iter()
                                    .enumerate()
                                    .filter(|(i, _)| tx_info.transaction.message().is_signer(*i))
                                    .map(|(_, pk)| bs58::encode(pk).into_string())
                                    .collect();

                                parsed_events.push(ParsedEvent {
                                    event_stream: stream.clone(),
                                    event_name: event_name.clone(),
                                    transaction_signature: bs58::encode(tx_info.signature)
                                        .into_string(),
                                    signers,
                                    data_json,
                                });
                            }
                            Err(e) => warn!("Borsh event deserialization failed: {}", e),
                        }
                    }
                }
            }
        }
        parsed_events
    }
}
