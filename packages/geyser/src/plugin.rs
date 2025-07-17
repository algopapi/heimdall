
use {
    crate::{
        event::*,
        filter::Filter,
        Config, PrometheusService, Publisher,
    },
    agave_geyser_plugin_interface::geyser_plugin_interface::{
        GeyserPlugin, GeyserPluginError as PluginError, ReplicaAccountInfoV3,
        ReplicaAccountInfoVersions, ReplicaTransactionInfoV2, ReplicaTransactionInfoVersions,
        Result as PluginResult, SlotStatus as PluginSlotStatus,
    },
    log::{debug, error, info, log_enabled},
    solana_pubkey::Pubkey,
    std::fmt::{Debug, Formatter},
    tokio::runtime::Runtime,
};

#[derive(Default)]
pub struct RedisPlugin {
    publisher: Option<Publisher>,
    filter: Option<Vec<Filter>>,
    prometheus: Option<PrometheusService>,
    runtime: Option<Runtime>,
}

impl Debug for RedisPlugin {
    fn fmt(&self, _: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl GeyserPlugin for RedisPlugin {
    fn name(&self) -> &'static str {
        "RedisPlugin"
    }

    fn on_load(&mut self, config_file: &str, _: bool) -> PluginResult<()> {
        if self.publisher.is_some() {
            return Err(PluginError::Custom("plugin already loaded".into()));
        }

        solana_logger::setup_with_default("info");
        info!(
            "Loading plugin {:?} from config_file {:?}",
            self.name(),
            config_file
        );
        let config = Config::read_from(config_file)?;

        info!("Creating Redis client connection");

        let client = config.redis_client().map_err(|error| {
            error!("Failed to create redis client: {error:?}");
            PluginError::Custom(Box::new(error))
        })?;
        info!("Created Redis client");

        let runtime = Runtime::new().map_err(|error| {
            error!("Failed to create tokio runtime: {error:?}");
            PluginError::Custom(Box::new(error))
        })?;

        let publisher = Publisher::new(client, &config);
        let prometheus = config
            .create_prometheus()
            .map_err(|error| PluginError::Custom(Box::new(error)))?;
        self.publisher = Some(publisher);
        self.filter = Some(config.filters.iter().map(Filter::new).collect());
        self.prometheus = prometheus;
        self.runtime = Some(runtime);

        Ok(())
    }

    fn on_unload(&mut self) {
        self.publisher = None;
        self.filter = None;
        if let Some(prometheus) = self.prometheus.take() {
            prometheus.shutdown();
        }
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_background();
        }
    }

    fn update_account(
        &self,
        account: ReplicaAccountInfoVersions,
        slot: u64,
        is_startup: bool,
    ) -> PluginResult<()> {
        let filters = self.unwrap_filters();
        if is_startup && filters.iter().all(|filter| !filter.publish_all_accounts) {
            return Ok(());
        }

        let info = Self::unwrap_update_account(account);
        let publisher = self.unwrap_publisher();
        for filter in filters {
            if !filter.update_account_stream.is_empty() {
                if !filter.wants_program(info.owner) && !filter.wants_account(info.pubkey) {
                    Self::log_ignore_account_update(info);
                    continue;
                }

                let event = UpdateAccountEvent {
                    slot,
                    pubkey: info.pubkey.to_vec(),
                    lamports: info.lamports,
                    owner: info.owner.to_vec(),
                    executable: info.executable,
                    rent_epoch: info.rent_epoch,
                    data: info.data.to_vec(),
                    write_version: info.write_version,
                    txn_signature: info.txn.map(|v| v.signature().as_ref().to_owned()),
                };

                let runtime = self.runtime.as_ref().expect("runtime is unavailable");
                runtime.block_on(async {
                    publisher
                        .update_account(event, filter.wrap_messages, &filter.update_account_stream)
                        .await
                        .map_err(|e| PluginError::AccountsUpdateError { msg: e.to_string() })
                })?;
            }
        }

        Ok(())
    }

    fn update_slot_status(
        &self,
        slot: u64,
        parent: Option<u64>,
        status: &PluginSlotStatus,
    ) -> PluginResult<()> {
        let publisher = self.unwrap_publisher();
        let value = SlotStatus::from(status.clone());
        for filter in self.unwrap_filters() {
            if !filter.slot_status_stream.is_empty() {
                let event = SlotStatusEvent {
                    slot,
                    parent: parent.unwrap_or(0),
                    status: value.into(),
                };

                let runtime = self.runtime.as_ref().expect("runtime is unavailable");
                runtime.block_on(async {
                    publisher
                        .update_slot_status(event, filter.wrap_messages, &filter.slot_status_stream)
                        .await
                        .map_err(|e| PluginError::AccountsUpdateError { msg: e.to_string() })
                })?;
            }
        }

        Ok(())
    }

    fn notify_transaction(
        &self,
        transaction: ReplicaTransactionInfoVersions,
        slot: u64,
    ) -> PluginResult<()> {
        let info = Self::unwrap_transaction(transaction);
        let publisher = self.unwrap_publisher();
        for filter in self.unwrap_filters() {
            if !filter.transaction_stream.is_empty() {
                let is_failed = info.transaction_status_meta.status.is_err();
                if (!filter.wants_vote_tx() && info.is_vote)
                    || (!filter.wants_failed_tx() && is_failed)
                {
                    debug!("Ignoring vote/failed transaction");
                    continue;
                }

                if !info
                    .transaction
                    .message()
                    .account_keys()
                    .iter()
                    .any(|pubkey| {
                        filter.wants_program(pubkey.as_ref())
                            || filter.wants_account(pubkey.as_ref())
                    })
                {
                    debug!("Ignoring transaction {:?}", info.signature);
                    continue;
                }

                let event = Self::build_transaction_event(slot, info);
                let runtime = self.runtime.as_ref().expect("runtime is unavailable");
                runtime.block_on(async {
                    publisher
                        .update_transaction(event, filter.wrap_messages, &filter.transaction_stream)
                        .await
                        .map_err(|e| PluginError::TransactionUpdateError { msg: e.to_string() })
                })?;
            }
        }

        Ok(())
    }

    fn account_data_notifications_enabled(&self) -> bool {
        let filters = self.unwrap_filters();
        filters
            .iter()
            .any(|filter| !filter.update_account_stream.is_empty())
    }

    fn transaction_notifications_enabled(&self) -> bool {
        let filters = self.unwrap_filters();
        filters
            .iter()
            .any(|filter| !filter.transaction_stream.is_empty())
    }
}

impl RedisPlugin {
    pub fn new() -> Self {
        Default::default()
    }

    fn unwrap_publisher(&self) -> &Publisher {
        self.publisher.as_ref().expect("publisher is unavailable")
    }

    fn unwrap_filters(&self) -> &Vec<Filter> {
        self.filter.as_ref().expect("filter is unavailable")
    }

    fn unwrap_update_account(account: ReplicaAccountInfoVersions) -> &ReplicaAccountInfoV3 {
        match account {
            ReplicaAccountInfoVersions::V0_0_1(_info) => {
                panic!("ReplicaAccountInfoVersions::V0_0_1 unsupported, please upgrade your Solana node.");
            }
            ReplicaAccountInfoVersions::V0_0_2(_info) => {
                panic!("ReplicaAccountInfoVersions::V0_0_2 unsupported, please upgrade your Solana node.");
            }
            ReplicaAccountInfoVersions::V0_0_3(info) => info,
        }
    }

    fn unwrap_transaction(
        transaction: ReplicaTransactionInfoVersions,
    ) -> &ReplicaTransactionInfoV2 {
        match transaction {
            ReplicaTransactionInfoVersions::V0_0_1(_info) => {
                panic!("ReplicaTransactionInfoVersions::V0_0_1 unsupported, please upgrade your Solana node.");
            }
            ReplicaTransactionInfoVersions::V0_0_2(info) => info,
        }
    }

    fn build_compiled_instruction(
        ix: &solana_message::compiled_instruction::CompiledInstruction,
    ) -> CompiledInstruction {
        CompiledInstruction {
            program_id_index: ix.program_id_index as u32,
            accounts: ix.clone().accounts.into_iter().map(|v| v as u32).collect(),
            data: ix.data.clone(),
        }
    }

    fn build_inner_instruction(
        ix: &solana_transaction_status::InnerInstruction,
    ) -> InnerInstruction {
        InnerInstruction {
            instruction: Some(Self::build_compiled_instruction(&ix.instruction)),
            stack_height: ix.stack_height,
        }
    }

    fn build_message_header(header: &solana_message::MessageHeader) -> MessageHeader {
        MessageHeader {
            num_required_signatures: header.num_required_signatures as u32,
            num_readonly_signed_accounts: header.num_readonly_signed_accounts as u32,
            num_readonly_unsigned_accounts: header.num_readonly_unsigned_accounts as u32,
        }
    }

    fn build_transaction_token_balance(
        transaction_token_account_balance: solana_transaction_status::TransactionTokenBalance,
    ) -> TransactionTokenBalance {
        TransactionTokenBalance {
            account_index: transaction_token_account_balance.account_index as u32,
            ui_token_account: Some(UiTokenAmount {
                ui_amount: transaction_token_account_balance.ui_token_amount.ui_amount,
                decimals: transaction_token_account_balance.ui_token_amount.decimals as u32,
                amount: transaction_token_account_balance.ui_token_amount.amount,
                ui_amount_string: transaction_token_account_balance
                    .ui_token_amount
                    .ui_amount_string,
            }),
            mint: transaction_token_account_balance.mint,
            owner: transaction_token_account_balance.owner,
        }
    }

    fn build_transaction_event(
        slot: u64,
        ReplicaTransactionInfoV2 {
            signature,
            is_vote,
            transaction,
            transaction_status_meta,
            index,
        }: &ReplicaTransactionInfoV2,
    ) -> TransactionEvent {
        TransactionEvent {
            is_vote: *is_vote,
            slot,
            index: *index as u64,
            signature: signature.as_ref().into(),
            transaction_status_meta: Some(TransactionStatusMeta {
                is_status_err: transaction_status_meta.status.is_err(),
                error_info: match &transaction_status_meta.status {
                    Err(e) => e.to_string(),
                    Ok(_) => "".to_owned(),
                },
                rewards: transaction_status_meta
                    .rewards
                    .clone()
                    .unwrap()
                    .into_iter()
                    .map(|x| Reward {
                        pubkey: x.pubkey,
                        lamports: x.lamports,
                        post_balance: x.post_balance,
                        reward_type: match x.reward_type {
                            Some(r) => r as i32,
                            None => 0,
                        },
                        commission: match x.commission {
                            Some(v) => v as u32,
                            None => 0,
                        },
                    })
                    .collect(),
                fee: transaction_status_meta.fee,
                log_messages: match &transaction_status_meta.log_messages {
                    Some(v) => v.to_owned(),
                    None => vec![],
                },
                inner_instructions: match &transaction_status_meta.inner_instructions {
                    Some(inners) => inners
                        .clone()
                        .into_iter()
                        .map(|inner| InnerInstructions {
                            index: inner.index as u32,
                            instructions: inner
                                .instructions
                                .iter()
                                .map(Self::build_inner_instruction)
                                .collect(),
                        })
                        .collect(),
                    None => vec![],
                },
                pre_balances: transaction_status_meta.pre_balances.clone(),
                post_balances: transaction_status_meta.post_balances.clone(),
                pre_token_balances: match &transaction_status_meta.pre_token_balances {
                    Some(v) => v
                        .clone()
                        .into_iter()
                        .map(Self::build_transaction_token_balance)
                        .collect(),
                    None => vec![],
                },
                post_token_balances: match &transaction_status_meta.post_token_balances {
                    Some(v) => v
                        .clone()
                        .into_iter()
                        .map(Self::build_transaction_token_balance)
                        .collect(),
                    None => vec![],
                },
            }),
            transaction: Some(SanitizedTransaction {
                message_hash: transaction.message_hash().to_bytes().into(),
                is_simple_vote_transaction: transaction.is_simple_vote_transaction(),
                message: Some(SanitizedMessage {
                    message_payload: Some(match transaction.message() {
                        solana_message::SanitizedMessage::Legacy(lv) => {
                            sanitized_message::MessagePayload::Legacy(LegacyLoadedMessage {
                                message: Some(LegacyMessage {
                                    header: Some(Self::build_message_header(&lv.message.header)),
                                    account_keys: lv
                                        .message
                                        .account_keys
                                        .clone()
                                        .into_iter()
                                        .map(|k| k.as_ref().into())
                                        .collect(),
                                    instructions: lv
                                        .message
                                        .instructions
                                        .iter()
                                        .map(Self::build_compiled_instruction)
                                        .collect(),
                                    recent_block_hash: lv.message.recent_blockhash.as_ref().into(),
                                }),
                                is_writable_account_cache: (0..(lv.account_keys().len() - 1))
                                    .map(|i: usize| lv.is_writable(i))
                                    .collect(),
                            })
                        }
                        solana_message::SanitizedMessage::V0(v0) => {
                            sanitized_message::MessagePayload::V0(V0LoadedMessage {
                                message: Some(V0Message {
                                    header: Some(Self::build_message_header(&v0.message.header)),
                                    account_keys: v0
                                        .message
                                        .account_keys
                                        .clone()
                                        .into_iter()
                                        .map(|k| k.as_ref().into())
                                        .collect(),
                                    recent_block_hash: v0.message.recent_blockhash.as_ref().into(),
                                    instructions: v0
                                        .message
                                        .instructions
                                        .iter()
                                        .map(Self::build_compiled_instruction)
                                        .collect(),
                                    address_table_lookup: v0
                                        .message
                                        .address_table_lookups
                                        .clone()
                                        .into_iter()
                                        .map(|vf| MessageAddressTableLookup {
                                            account_key: vf.account_key.as_ref().into(),
                                            writable_indexes: vf
                                                .writable_indexes
                                                .iter()
                                                .map(|x| *x as u32)
                                                .collect(),
                                            readonly_indexes: vf
                                                .readonly_indexes
                                                .iter()
                                                .map(|x| *x as u32)
                                                .collect(),
                                        })
                                        .collect(),
                                }),
                                loaded_adresses: Some(LoadedAddresses {
                                    writable: v0
                                        .loaded_addresses
                                        .writable
                                        .clone()
                                        .into_iter()
                                        .map(|x| x.as_ref().into())
                                        .collect(),
                                    readonly: v0
                                        .loaded_addresses
                                        .readonly
                                        .clone()
                                        .into_iter()
                                        .map(|x| x.as_ref().into())
                                        .collect(),
                                }),
                                is_writable_account_cache: (0..(v0.account_keys().len() - 1))
                                    .map(|i: usize| v0.is_writable(i))
                                    .collect(),
                            })
                        }
                    }),
                }),
                signatures: transaction
                    .signatures()
                    .iter()
                    .copied()
                    .map(|x| x.as_ref().into())
                    .collect(),
            }),
        }
    }

    fn log_ignore_account_update(info: &ReplicaAccountInfoV3) {
        if log_enabled!(::log::Level::Debug) {
            match <&[u8; 32]>::try_from(info.owner) {
                Ok(key) => debug!(
                    "Ignoring update for account key: {:?}",
                    Pubkey::new_from_array(*key)
                ),
                // Err should never happen because wants_account_key only returns false if the input is &[u8; 32]
                Err(_err) => debug!("Ignoring update for account key: {:?}", info.owner),
            };
        }
    }
}
