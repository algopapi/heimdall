use {
    crate::{
        error::PluginError,
        model::{ParsedAccount, ParsedEvent, RedisMessage},
    },
    log::{debug, error, info},
    redis::AsyncCommands,
    std::thread::{self, JoinHandle},
    tokio::runtime::Runtime,
    tokio::sync::mpsc::{self, Sender},
};

pub struct RedisPublisher {
    sender: Sender<RedisMessage>,
    worker_handle: Option<JoinHandle<()>>,
}

impl RedisPublisher {
    pub fn new(redis_url: String) -> Result<Self, PluginError> {
        let (sender, mut receiver) = mpsc::channel::<RedisMessage>(10000);

        let worker_handle = thread::spawn(move || {
            let rt = Runtime::new().expect("Failed to create Tokio runtime");
            rt.block_on(async move {
                info!("Redis worker thread started.");
                let client = redis::Client::open(redis_url).unwrap();
                let mut conn = client.get_async_connection().await.unwrap();

                while let Some(msg) = receiver.recv().await {
                    let result: redis::RedisResult<()> = match msg {
                        RedisMessage::AccountUpdate {
                            stream,
                            account_pubkey,
                            data,
                            slot,
                        } => {
                            let payload = serde_json::to_string(&data).unwrap_or_default();
                            conn.xadd(
                                stream,
                                "*",
                                &[
                                    ("pubkey", &account_pubkey),
                                    ("data", &payload),
                                    ("slot", &slot.to_string()),
                                ],
                            )
                            .await
                        }
                        RedisMessage::Event {
                            stream,
                            signature,
                            signers,
                            data,
                            slot,
                        } => {
                            let payload = serde_json::to_string(&data).unwrap_or_default();
                            let signers_json = serde_json::to_string(&signers).unwrap_or_default();
                            conn.xadd(
                                stream,
                                "*",
                                &[
                                    ("signature", &signature),
                                    ("signers", &signers_json),
                                    ("data", &payload),
                                    ("slot", &slot.to_string()),
                                ],
                            )
                            .await
                        }
                    };

                    if let Err(e) = result {
                        error!("Failed to publish to Redis stream: {}", e);
                    }
                }
                info!("Redis worker thread shutting down.");
            });
        });

        Ok(Self {
            sender,
            worker_handle: Some(worker_handle),
        })
    }

    pub fn publish_account_update(
        &self,
        parsed_account: ParsedAccount,
        slot: u64,
    ) -> Result<(), PluginError> {
        let msg = RedisMessage::AccountUpdate {
            stream: parsed_account.account_stream,
            account_pubkey: parsed_account.account_pubkey,
            data: parsed_account.data_json,
            slot,
        };
        self.sender
            .try_send(msg)
            .map_err(|e| PluginError::ChannelSendError(e.to_string()))
    }

    pub fn publish_event(&self, parsed_event: ParsedEvent, slot: u64) -> Result<(), PluginError> {
        let msg = RedisMessage::Event {
            stream: parsed_event.event_stream,
            signature: parsed_event.transaction_signature,
            signers: parsed_event.signers,
            data: parsed_event.data_json,
            slot,
        };
        self.sender
            .try_send(msg)
            .map_err(|e| PluginError::ChannelSendError(e.to_string()))
    }
}

impl Drop for RedisPublisher {
    fn drop(&mut self) {
        info!("Shutting down Redis publisher.");
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
        info!("Redis publisher shut down complete.");
    }
}
