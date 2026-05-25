//! PumpSwap instruction parser
//!
//! Parse PumpSwap instructions using discriminator pattern matching

use super::program_ids;
use super::utils::*;
use crate::core::events::*;
use solana_sdk::{pubkey::Pubkey, signature::Signature};

/// PumpSwap instruction discriminator constants (from pump_amm.json)
pub mod discriminators {
    /// buy: Buy tokens with quote (SOL)
    pub const BUY: [u8; 8] = [102, 6, 61, 18, 1, 218, 235, 234];
    /// sell: Sell tokens for quote (SOL)
    pub const SELL: [u8; 8] = [51, 230, 133, 164, 1, 127, 131, 173];
    /// create_pool: Create a new AMM pool
    pub const CREATE_POOL: [u8; 8] = [233, 146, 209, 142, 207, 104, 64, 188];
    /// buy_exact_quote_in: Buy tokens with exact quote amount
    pub const BUY_EXACT_QUOTE_IN: [u8; 8] = [198, 46, 21, 82, 180, 217, 232, 112];
    /// deposit: Add liquidity to pool
    pub const DEPOSIT: [u8; 8] = [242, 35, 198, 137, 82, 225, 242, 182];
    /// withdraw: Remove liquidity from pool
    pub const WITHDRAW: [u8; 8] = [183, 18, 70, 156, 148, 109, 161, 34];
}

/// Pump AMM Program ID
pub const PROGRAM_ID_PUBKEY: Pubkey = program_ids::PUMPSWAP_PROGRAM_ID;

/// Main PumpSwap instruction parser
///
/// Parses main instructions to extract account information.
/// This will be merged with inner instruction events to form complete events.
pub fn parse_instruction(
    instruction_data: &[u8],
    accounts: &[Pubkey],
    signature: Signature,
    slot: u64,
    tx_index: u64,
    block_time_us: Option<i64>,
) -> Option<DexEvent> {
    // Check minimum data length for discriminator
    if instruction_data.len() < 8 {
        return None;
    }

    // Extract 8-byte discriminator
    let discriminator: [u8; 8] = instruction_data[0..8].try_into().ok()?;
    let data = &instruction_data[8..];

    // Route based on discriminator
    match discriminator {
        discriminators::BUY => {
            parse_buy_instruction(data, accounts, signature, slot, tx_index, block_time_us)
        }
        discriminators::BUY_EXACT_QUOTE_IN => parse_buy_exact_quote_in_instruction(
            data,
            accounts,
            signature,
            slot,
            tx_index,
            block_time_us,
        ),
        discriminators::SELL => {
            parse_sell_instruction(data, accounts, signature, slot, tx_index, block_time_us)
        }
        discriminators::CREATE_POOL => {
            parse_create_pool_instruction(data, accounts, signature, slot, tx_index, block_time_us)
        }
        discriminators::DEPOSIT => {
            parse_deposit_instruction(data, accounts, signature, slot, tx_index, block_time_us)
        }
        discriminators::WITHDRAW => {
            parse_withdraw_instruction(data, accounts, signature, slot, tx_index, block_time_us)
        }
        _ => None,
    }
}

/// Parse buy instruction
///
/// Account indices (from pump_amm.json IDL), 23 个固定账户:
/// 0 pool, 1 user, 2 global_config, 3 base_mint, 4 quote_mint,
/// 5 user_base_token_account, 6 user_quote_token_account,
/// 7 pool_base_token_account, 8 pool_quote_token_account,
/// 9 protocol_fee_recipient, 10 protocol_fee_recipient_token_account,
/// 11 base_token_program, 12 quote_token_program,
/// 13 system_program, 14 associated_token_program, 15 event_authority, 16 program,
/// 17 coin_creator_vault_ata, 18 coin_creator_vault_authority,
/// 19 global_volume_accumulator, 20 user_volume_accumulator, 21 fee_config, 22 fee_program.
#[allow(dead_code)]
fn parse_buy_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    signature: Signature,
    slot: u64,
    tx_index: u64,
    block_time_us: Option<i64>,
) -> Option<DexEvent> {
    if accounts.len() < 13 {
        return None;
    }

    // Parse args: base_amount_out (u64), max_quote_amount_in (u64)
    // NOTE: buy instruction has TOKEN first, SOL second
    let (base_amount, quote_amount) = if data.len() >= 16 {
        (read_u64_le(data, 0).unwrap_or(0), read_u64_le(data, 8).unwrap_or(0))
    } else {
        (0, 0)
    };

    let metadata = create_metadata(signature, slot, tx_index, block_time_us.unwrap_or_default(), 0);

    let mut ev = PumpSwapBuyEvent {
        metadata,
        pool: get_account(accounts, 0).unwrap_or_default(),
        user: get_account(accounts, 1).unwrap_or_default(),
        base_mint: get_account(accounts, 3).unwrap_or_default(),
        quote_mint: get_account(accounts, 4).unwrap_or_default(),
        user_base_token_account: get_account(accounts, 5).unwrap_or_default(),
        user_quote_token_account: get_account(accounts, 6).unwrap_or_default(),
        pool_base_token_account: get_account(accounts, 7).unwrap_or_default(),
        pool_quote_token_account: get_account(accounts, 8).unwrap_or_default(),
        protocol_fee_recipient: get_account(accounts, 9).unwrap_or_default(),
        protocol_fee_recipient_token_account: get_account(accounts, 10).unwrap_or_default(),
        base_token_program: get_account(accounts, 11).unwrap_or_default(),
        quote_token_program: get_account(accounts, 12).unwrap_or_default(),
        base_amount_out: base_amount,
        max_quote_amount_in: quote_amount,
        ..Default::default()
    };
    if accounts.len() >= 19 {
        ev.coin_creator_vault_ata = get_account(accounts, 17).unwrap_or_default();
        ev.coin_creator_vault_authority = get_account(accounts, 18).unwrap_or_default();
    }
    ev.all_instruction_accounts = accounts.to_vec();
    Some(DexEvent::PumpSwapBuy(ev))
}

/// Parse buy_exact_quote_in instruction
///
/// IMPORTANT: Parameter order is DIFFERENT from buy instruction!
/// - buy: base_amount_out (token) first, max_quote_amount_in (SOL) second
/// - buy_exact_quote_in: spendable_quote_in (SOL) first, min_base_amount_out (token) second
///
/// Account indices: 与 buy 相同，共 23 个 (0-22)，见 parse_buy_instruction 注释。
#[allow(dead_code)]
fn parse_buy_exact_quote_in_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    signature: Signature,
    slot: u64,
    tx_index: u64,
    block_time_us: Option<i64>,
) -> Option<DexEvent> {
    if accounts.len() < 13 {
        return None;
    }

    // Parse args: spendable_quote_in (u64), min_base_amount_out (u64)
    // NOTE: buy_exact_quote_in has SOL first, TOKEN second (reversed from buy!)
    let (quote_amount, base_amount) = if data.len() >= 16 {
        (read_u64_le(data, 0).unwrap_or(0), read_u64_le(data, 8).unwrap_or(0))
    } else {
        (0, 0)
    };

    let metadata = create_metadata(signature, slot, tx_index, block_time_us.unwrap_or_default(), 0);

    let mut ev = PumpSwapBuyEvent {
        metadata,
        pool: get_account(accounts, 0).unwrap_or_default(),
        user: get_account(accounts, 1).unwrap_or_default(),
        base_mint: get_account(accounts, 3).unwrap_or_default(),
        quote_mint: get_account(accounts, 4).unwrap_or_default(),
        user_base_token_account: get_account(accounts, 5).unwrap_or_default(),
        user_quote_token_account: get_account(accounts, 6).unwrap_or_default(),
        pool_base_token_account: get_account(accounts, 7).unwrap_or_default(),
        pool_quote_token_account: get_account(accounts, 8).unwrap_or_default(),
        protocol_fee_recipient: get_account(accounts, 9).unwrap_or_default(),
        protocol_fee_recipient_token_account: get_account(accounts, 10).unwrap_or_default(),
        base_token_program: get_account(accounts, 11).unwrap_or_default(),
        quote_token_program: get_account(accounts, 12).unwrap_or_default(),
        base_amount_out: base_amount,
        max_quote_amount_in: quote_amount,
        ..Default::default()
    };
    if accounts.len() >= 19 {
        ev.coin_creator_vault_ata = get_account(accounts, 17).unwrap_or_default();
        ev.coin_creator_vault_authority = get_account(accounts, 18).unwrap_or_default();
    }
    ev.all_instruction_accounts = accounts.to_vec();
    Some(DexEvent::PumpSwapBuy(ev))
}

/// Parse sell instruction
///
/// Account indices (from pump_amm.json IDL), 21 个固定账户:
/// 0 pool, 1 user, 2 global_config, 3 base_mint, 4 quote_mint,
/// 5 user_base_token_account, 6 user_quote_token_account,
/// 7 pool_base_token_account, 8 pool_quote_token_account,
/// 9 protocol_fee_recipient, 10 protocol_fee_recipient_token_account,
/// 11 base_token_program, 12 quote_token_program,
/// 13 system_program, 14 associated_token_program, 15 event_authority, 16 program,
/// 17 coin_creator_vault_ata, 18 coin_creator_vault_authority,
/// 19 fee_config, 20 fee_program.
#[allow(dead_code)]
fn parse_sell_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    signature: Signature,
    slot: u64,
    tx_index: u64,
    block_time_us: Option<i64>,
) -> Option<DexEvent> {
    if accounts.len() < 13 {
        return None;
    }

    // Parse args: base_amount_in (u64), min_quote_amount_out (u64)
    let (base_amount, quote_amount) = if data.len() >= 16 {
        (read_u64_le(data, 0).unwrap_or(0), read_u64_le(data, 8).unwrap_or(0))
    } else {
        (0, 0)
    };

    let metadata = create_metadata(signature, slot, tx_index, block_time_us.unwrap_or_default(), 0);

    let mut ev = PumpSwapSellEvent {
        metadata,
        pool: get_account(accounts, 0).unwrap_or_default(),
        user: get_account(accounts, 1).unwrap_or_default(),
        base_mint: get_account(accounts, 3).unwrap_or_default(),
        quote_mint: get_account(accounts, 4).unwrap_or_default(),
        user_base_token_account: get_account(accounts, 5).unwrap_or_default(),
        user_quote_token_account: get_account(accounts, 6).unwrap_or_default(),
        pool_base_token_account: get_account(accounts, 7).unwrap_or_default(),
        pool_quote_token_account: get_account(accounts, 8).unwrap_or_default(),
        protocol_fee_recipient: get_account(accounts, 9).unwrap_or_default(),
        protocol_fee_recipient_token_account: get_account(accounts, 10).unwrap_or_default(),
        base_token_program: get_account(accounts, 11).unwrap_or_default(),
        quote_token_program: get_account(accounts, 12).unwrap_or_default(),
        base_amount_in: base_amount,
        min_quote_amount_out: quote_amount,
        ..Default::default()
    };
    if accounts.len() >= 19 {
        ev.coin_creator_vault_ata = get_account(accounts, 17).unwrap_or_default();
        ev.coin_creator_vault_authority = get_account(accounts, 18).unwrap_or_default();
    }
    ev.all_instruction_accounts = accounts.to_vec();
    Some(DexEvent::PumpSwapSell(ev))
}

/// Parse create_pool instruction
#[allow(dead_code)]
fn parse_create_pool_instruction(
    _data: &[u8],
    accounts: &[Pubkey],
    signature: Signature,
    slot: u64,
    tx_index: u64,
    block_time_us: Option<i64>,
) -> Option<DexEvent> {
    if accounts.len() < 5 {
        return None;
    }

    let metadata = create_metadata(signature, slot, tx_index, block_time_us.unwrap_or_default(), 0);

    Some(DexEvent::PumpSwapCreatePool(PumpSwapCreatePoolEvent {
        metadata,
        creator: get_account(accounts, 0).unwrap_or_default(),
        base_mint: get_account(accounts, 2).unwrap_or_default(),
        quote_mint: get_account(accounts, 3).unwrap_or_default(),
        ..Default::default()
    }))
}

/// Parse deposit (add liquidity) instruction
#[allow(dead_code)]
fn parse_deposit_instruction(
    _data: &[u8],
    accounts: &[Pubkey],
    signature: Signature,
    slot: u64,
    tx_index: u64,
    block_time_us: Option<i64>,
) -> Option<DexEvent> {
    if accounts.len() < 8 {
        return None;
    }

    let metadata = create_metadata(signature, slot, tx_index, block_time_us.unwrap_or_default(), 0);

    Some(DexEvent::PumpSwapLiquidityAdded(PumpSwapLiquidityAdded {
        metadata,
        pool: get_account(accounts, 0).unwrap_or_default(),
        user: get_account(accounts, 1).unwrap_or_default(),
        user_base_token_account: get_account(accounts, 4).unwrap_or_default(),
        user_quote_token_account: get_account(accounts, 5).unwrap_or_default(),
        user_pool_token_account: get_account(accounts, 6).unwrap_or_default(),
        ..Default::default()
    }))
}

/// Parse withdraw (remove liquidity) instruction
#[allow(dead_code)]
fn parse_withdraw_instruction(
    _data: &[u8],
    accounts: &[Pubkey],
    signature: Signature,
    slot: u64,
    tx_index: u64,
    block_time_us: Option<i64>,
) -> Option<DexEvent> {
    if accounts.len() < 8 {
        return None;
    }

    let metadata = create_metadata(signature, slot, tx_index, block_time_us.unwrap_or_default(), 0);

    Some(DexEvent::PumpSwapLiquidityRemoved(PumpSwapLiquidityRemoved {
        metadata,
        pool: get_account(accounts, 0).unwrap_or_default(),
        user: get_account(accounts, 1).unwrap_or_default(),
        user_base_token_account: get_account(accounts, 4).unwrap_or_default(),
        user_quote_token_account: get_account(accounts, 5).unwrap_or_default(),
        user_pool_token_account: get_account(accounts, 6).unwrap_or_default(),
        ..Default::default()
    }))
}
