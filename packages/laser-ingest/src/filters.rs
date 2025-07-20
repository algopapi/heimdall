use helius_laserstream::grpc::{
    SubscribeRequest,
    SubscribeRequestFilterAccounts,
    SubscribeRequestFilterTransactions,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Plain struct that represents the dynamic filter configuration we load from Supabase.
/// It derives Serialize / Deserialize so we can load it directly from JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FilterSet {
    pub programs: Vec<String>,
    pub accounts: Vec<String>,
    #[serde(default)]
    pub account_required: Vec<String>,
    #[serde(default)]
    pub publish_all: bool,
    #[serde(default)]
    pub include_votes: bool,
    #[serde(default)]
    pub include_failed: bool,
}

impl FilterSet {
    /// Transform into a Helius `SubscribeRequest` ready to be sent to LaserStream.
    pub fn into_subscribe_request(self) -> SubscribeRequest {
        let mut request = SubscribeRequest::default();

        // ------------------------------------------------
        // Account filters (for `Account` updates)
        // ------------------------------------------------
        if !self.accounts.is_empty() {
            let mut map = HashMap::new();
            map.insert(
                "dynamic".to_owned(),
                SubscribeRequestFilterAccounts {
                    account: self.accounts.clone(),
                    ..Default::default()
                },
            );
            request.accounts = map;
        }

        // ------------------------------------------------
        // Transaction filters (for `Transaction` updates)
        // ------------------------------------------------
        if !self.publish_all || !self.programs.is_empty() {
            let mut map = HashMap::new();
            map.insert(
                "dynamic".to_owned(),
                SubscribeRequestFilterTransactions {
                    account_include: self.programs.clone(),
                    account_required: self.account_required.clone(),
                    vote: Some(self.include_votes),
                    failed: Some(self.include_failed),
                    ..Default::default()
                },
            );
            request.transactions = map;
        }

        // ------------------------------------------------
        // Return built request
        // ------------------------------------------------
        request
    }
} 