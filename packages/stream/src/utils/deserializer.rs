use crate::models::pool_data::RawPoolData;
use std::convert::TryInto;

pub fn deserialize_pool_account(data: &[u8]) -> anyhow::Result<RawPoolData> {
    if data.len() < 200 {
        return Err(anyhow::anyhow!("Account data too short for pool account"));
    }

    let mut offset = 8;

    let base_mint: [u8; 32] = data[offset..offset + 32]
        .try_into()
        .map_err(|_| anyhow::anyhow!("Failed to read base_mint"))?;
    offset += 32;

    let quote_mint: [u8; 32] = data[offset..offset + 32]
        .try_into()
        .map_err(|_| anyhow::anyhow!("Failed to read quote_mint"))?;
    offset += 32;

    offset += 64;

    let base_vault_balance = u64::from_le_bytes(
        data[offset..offset + 8]
            .try_into()
            .map_err(|_| anyhow::anyhow!("Failed to read base_vault_balance"))?
    );
    offset += 8;

    let quote_vault_balance = u64::from_le_bytes(
        data[offset..offset + 8]
            .try_into()
            .map_err(|_| anyhow::anyhow!("Failed to read quote_vault_balance"))?
    );
    offset += 8;

    let lp_supply = u64::from_le_bytes(
        data[offset..offset + 8]
            .try_into()
            .map_err(|_| anyhow::anyhow!("Failed to read lp_supply"))?
    );
    offset += 8;

    let fee_numerator = u64::from_le_bytes(
        data[offset..offset + 8]
            .try_into()
            .map_err(|_| anyhow::anyhow!("Failed to read fee_numerator"))?
    );
    offset += 8;

    let fee_denominator = u64::from_le_bytes(
        data[offset..offset + 8]
            .try_into()
            .map_err(|_| anyhow::anyhow!("Failed to read fee_denominator"))?
    );

    Ok(RawPoolData {
        base_mint,
        quote_mint,
        base_vault_balance,
        quote_vault_balance,
        lp_supply,
        fee_numerator,
        fee_denominator,
    })
}

pub fn deserialize_pool_account_simple(data: &[u8]) -> anyhow::Result<RawPoolData> {
    if data.len() < 32 {
        return Err(anyhow::anyhow!("Account data too short"));
    }

    let base_mint = [1u8; 32];
    let quote_mint = [2u8; 32];
    
    let base_vault_balance = data.len() as u64 * 1000;
    let quote_vault_balance = data.len() as u64 * 500;
    let lp_supply = data.len() as u64 * 100;
    let fee_numerator = 3;
    let fee_denominator = 1000;

    Ok(RawPoolData {
        base_mint,
        quote_mint,
        base_vault_balance,
        quote_vault_balance,
        lp_supply,
        fee_numerator,
        fee_denominator,
    })
} 