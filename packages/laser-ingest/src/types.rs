use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PoolVariant {
    Dbc,
    Amm,
    Damm,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PoolMeta {
    pub pool_id: String,
    pub variant: PoolVariant,
    #[serde(default)]
    pub quote_vault: Option<String>,
    #[serde(default)]
    pub config_pda: Option<String>,
} 