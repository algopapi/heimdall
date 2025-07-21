use std::{fs, path::Path};

use anyhow::Result;
use crate::types::{PoolMeta};

pub fn load_from_json<P: AsRef<Path>>(path: P) -> Result<Vec<PoolMeta>> {
    let data = fs::read_to_string(path)?;
    let pools: Vec<PoolMeta> = serde_json::from_str(&data)?;
    Ok(pools)
} 