//! 所有具体的事件类型定义
//!
//! 基于您提供的回调事件列表，定义所有需要的具体事件类型

// use prost_types::Timestamp;
use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signature::Signature};

/// 基础元数据 - 所有事件共享的字段
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventMetadata {
    pub signature: Signature,
    pub slot: u64,
    pub tx_index: u64, // 交易在slot中的索引，参考solana-streamer
    pub block_time_us: i64,
    pub grpc_recv_us: i64,
    /// Transaction's recent blockhash as base58 string, when available.
    #[serde(default)]
    pub recent_blockhash: Option<String>,
}

/// Block Meta Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockMetaEvent {
    pub metadata: EventMetadata,
}

/// Bonk Pool Create Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonkPoolCreateEvent {
    pub metadata: EventMetadata,
    pub base_mint_param: BaseMintParam,
    pub pool_state: Pubkey,
    pub creator: Pubkey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseMintParam {
    pub symbol: String,
    pub name: String,
    pub uri: String,
    pub decimals: u8,
}

/// Bonk Trade Event
#[cfg_attr(feature = "parse-borsh", derive(BorshDeserialize))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonkTradeEvent {
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub metadata: EventMetadata,

    // === Borsh 序列化字段（从 inner instruction data 读取）===
    pub pool_state: Pubkey, // 32 bytes
    pub user: Pubkey,       // 32 bytes
    pub amount_in: u64,     // 8 bytes
    pub amount_out: u64,    // 8 bytes
    pub is_buy: bool,       // 1 byte

    // === 非 Borsh 字段（派生字段）===
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub trade_direction: TradeDirection,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub exact_in: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum TradeDirection {
    #[default]
    Buy,
    Sell,
}

/// Bonk Migrate AMM Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonkMigrateAmmEvent {
    pub metadata: EventMetadata,
    pub old_pool: Pubkey,
    pub new_pool: Pubkey,
    pub user: Pubkey,
    pub liquidity_amount: u64,
}

/// PumpFun Trade Event - 基于官方IDL定义
///
/// 字段来源标记:
/// - [EVENT]: 来自原始IDL事件定义，由程序日志直接解析获得
/// - [INSTRUCTION]: 来自指令解析，用于补充事件缺失的上下文信息
#[derive(Debug, Clone, Serialize, Deserialize, Default, BorshDeserialize)]
pub struct PumpFunTradeEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,

    // === IDL TradeEvent 事件字段（Borsh 序列化字段，按顺序）===
    pub mint: Pubkey,
    pub sol_amount: u64,
    pub token_amount: u64,
    pub is_buy: bool,
    #[borsh(skip)]
    pub is_created_buy: bool, // 由外层逻辑设置，不在 Borsh 数据中
    pub user: Pubkey,
    pub timestamp: i64,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub fee_recipient: Pubkey,
    pub fee_basis_points: u64,
    pub fee: u64,
    pub creator: Pubkey,
    pub creator_fee_basis_points: u64,
    pub creator_fee: u64,
    pub track_volume: bool,
    pub total_unclaimed_tokens: u64,
    pub total_claimed_tokens: u64,
    pub current_sol_volume: u64,
    pub last_update_timestamp: i64,
    /// Instruction name: "buy" | "sell" | "buy_exact_sol_in"
    pub ix_name: String,
    /// 与链上 / Explorer `tradeEvent` 中 `mayhemMode` 一致（gRPC 日志解析填充；勿用 fee 地址推断）。
    pub mayhem_mode: bool,
    /// Cashback fee basis points (PUMP_CASHBACK_README)
    pub cashback_fee_basis_points: u64,
    /// Cashback amount (PUMP_CASHBACK_README)
    pub cashback: u64,
    /// 是否返现代币（由 cashback_fee_basis_points > 0 推导，供 sol-trade-sdk 等构造 sell 指令用）
    #[borsh(skip)]
    pub is_cashback_coin: bool,

    // === Instruction parameter fields (reserved for future use, DO NOT delete) ===
    // pub amount: u64,                     // buy/sell.args.amount
    // pub max_sol_cost: u64,               // buy.args.maxSolCost
    // pub min_sol_output: u64,             // sell.args.minSolOutput

    // === 指令账户字段 (从指令账户填充，不在 Borsh 数据中) ===
    // pub global: Pubkey,                  // 0
    #[borsh(skip)]
    pub bonding_curve: Pubkey, // 3
    #[borsh(skip)]
    pub associated_bonding_curve: Pubkey, // 4
    // pub associated_user: Pubkey,         // 5
    #[borsh(skip)]
    pub token_program: Pubkey, // sell - 9 / buy - 8
    #[borsh(skip)]
    pub creator_vault: Pubkey, // sell - 8 / buy - 9
    /// 第 17 个指令账户 (index 16)，区块浏览器显示为 "Account"，部分 buy/sell 会传入
    #[borsh(skip)]
    pub account: Option<Pubkey>,
}

/// PumpFun Migrate Event
#[derive(Debug, Clone, Serialize, Deserialize, Default, BorshDeserialize)]
pub struct PumpFunMigrateEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    // Borsh 序列化字段（按顺序）
    pub user: Pubkey,
    pub mint: Pubkey,
    pub mint_amount: u64,
    pub sol_amount: u64,
    pub pool_migration_fee: u64,
    pub bonding_curve: Pubkey,
    pub timestamp: i64,
    pub pool: Pubkey,
    // === 额外账户信息（用于指令解析，暂时注释，以后可能会用到，AI不要删除） ===
    // pub global: Pubkey,
    // pub withdraw_authority: Pubkey,
    // pub associated_bonding_curve: Pubkey,
    // pub pump_amm: Pubkey,
    // pub pool_authority: Pubkey,
    // pub pool_authority_mint_account: Pubkey,
    // pub pool_authority_wsol_account: Pubkey,
    // pub amm_global_config: Pubkey,
    // pub wsol_mint: Pubkey,
    // pub lp_mint: Pubkey,
    // pub user_pool_token_account: Pubkey,
    // pub pool_base_token_account: Pubkey,
    // pub pool_quote_token_account: Pubkey,
}

/// PumpFun Create Token Event - Based on IDL CreateEvent definition
#[derive(Debug, Clone, Serialize, Deserialize, Default, BorshDeserialize)]
pub struct PumpFunCreateTokenEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    // IDL CreateEvent 字段（Borsh 序列化字段，按顺序）
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub mint: Pubkey,
    pub bonding_curve: Pubkey,
    pub user: Pubkey,
    pub creator: Pubkey,
    pub timestamp: i64,
    pub virtual_token_reserves: u64,
    pub virtual_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub token_total_supply: u64,

    pub token_program: Pubkey,
    pub is_mayhem_mode: bool,
    /// Cashback 是否开启 (IDL CreateEvent.is_cashback_enabled)
    pub is_cashback_enabled: bool,
}

/// PumpFun Create V2 Token Event (SPL-22 / Mayhem Mode)
/// 与 solana-streamer 对齐；指令解析时从 accounts 0..15 填充。
#[derive(Debug, Clone, Serialize, Deserialize, Default, BorshDeserialize)]
pub struct PumpFunCreateV2TokenEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub mint: Pubkey,
    pub bonding_curve: Pubkey,
    pub user: Pubkey,
    pub creator: Pubkey,
    pub timestamp: i64,
    pub virtual_token_reserves: u64,
    pub virtual_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub token_total_supply: u64,
    pub token_program: Pubkey,
    pub is_mayhem_mode: bool,
    pub is_cashback_enabled: bool,
    #[borsh(skip)]
    pub mint_authority: Pubkey,
    #[borsh(skip)]
    pub associated_bonding_curve: Pubkey,
    #[borsh(skip)]
    pub global: Pubkey,
    #[borsh(skip)]
    pub system_program: Pubkey,
    #[borsh(skip)]
    pub associated_token_program: Pubkey,
    #[borsh(skip)]
    pub mayhem_program_id: Pubkey,
    #[borsh(skip)]
    pub global_params: Pubkey,
    #[borsh(skip)]
    pub sol_vault: Pubkey,
    #[borsh(skip)]
    pub mayhem_state: Pubkey,
    #[borsh(skip)]
    pub mayhem_token_vault: Pubkey,
    #[borsh(skip)]
    pub event_authority: Pubkey,
    #[borsh(skip)]
    pub program: Pubkey,
    /// 同笔交易中后续 Pump Buy 的账户 #2（或 trade 日志中的 fee recipient）；由 `pumpfun_fee_enrich` 回填。
    #[borsh(skip)]
    pub observed_fee_recipient: Pubkey,
}

/// PumpSwap Trade Event - Unified trade event from IDL TradeEvent
/// Produced by: buy, sell, buy_exact_sol_in instructions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PumpSwapTradeEvent {
    pub metadata: EventMetadata,
    // === IDL TradeEvent fields ===
    pub mint: Pubkey,
    pub sol_amount: u64,
    pub token_amount: u64,
    pub is_buy: bool,
    pub user: Pubkey,
    pub timestamp: i64,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub fee_recipient: Pubkey,
    pub fee_basis_points: u64,
    pub fee: u64,
    pub creator: Pubkey,
    pub creator_fee_basis_points: u64,
    pub creator_fee: u64,
    pub track_volume: bool,
    pub total_unclaimed_tokens: u64,
    pub total_claimed_tokens: u64,
    pub current_sol_volume: u64,
    pub last_update_timestamp: i64,
    pub ix_name: String, // "buy" | "sell" | "buy_exact_sol_in"
}

/// PumpSwap Buy Event
#[derive(Debug, Clone, Serialize, Deserialize, Default, BorshDeserialize)]
pub struct PumpSwapBuyEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub timestamp: i64,
    pub base_amount_out: u64,
    pub max_quote_amount_in: u64,
    pub user_base_token_reserves: u64,
    pub user_quote_token_reserves: u64,
    pub pool_base_token_reserves: u64,
    pub pool_quote_token_reserves: u64,
    pub quote_amount_in: u64,
    pub lp_fee_basis_points: u64,
    pub lp_fee: u64,
    pub protocol_fee_basis_points: u64,
    pub protocol_fee: u64,
    pub quote_amount_in_with_lp_fee: u64,
    pub user_quote_amount_in: u64,
    pub pool: Pubkey,
    pub user: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub protocol_fee_recipient: Pubkey,
    pub protocol_fee_recipient_token_account: Pubkey,
    pub coin_creator: Pubkey,
    pub coin_creator_fee_basis_points: u64,
    pub coin_creator_fee: u64,
    pub track_volume: bool,
    pub total_unclaimed_tokens: u64,
    pub total_claimed_tokens: u64,
    pub current_sol_volume: u64,
    pub last_update_timestamp: i64,
    /// Minimum base token amount expected (new field from IDL update)
    pub min_base_amount_out: u64,
    /// Instruction name (new field from IDL update)
    pub ix_name: String,
    /// Cashback fee basis points (PUMP_CASHBACK_README)
    pub cashback_fee_basis_points: u64,
    /// Cashback amount (PUMP_CASHBACK_README)
    pub cashback: u64,

    // === 额外的信息 ===
    #[borsh(skip)]
    pub is_pump_pool: bool,

    // === 额外账户信息 (from instruction accounts, not event data) ===
    #[borsh(skip)]
    pub base_mint: Pubkey,
    #[borsh(skip)]
    pub quote_mint: Pubkey,
    #[borsh(skip)]
    pub pool_base_token_account: Pubkey,
    #[borsh(skip)]
    pub pool_quote_token_account: Pubkey,
    #[borsh(skip)]
    pub coin_creator_vault_ata: Pubkey,
    #[borsh(skip)]
    pub coin_creator_vault_authority: Pubkey,
    #[borsh(skip)]
    pub base_token_program: Pubkey,
    #[borsh(skip)]
    pub quote_token_program: Pubkey,
}

/// PumpSwap Sell Event
#[derive(Debug, Clone, Serialize, Deserialize, Default, BorshDeserialize)]
pub struct PumpSwapSellEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub timestamp: i64,
    pub base_amount_in: u64,
    pub min_quote_amount_out: u64,
    pub user_base_token_reserves: u64,
    pub user_quote_token_reserves: u64,
    pub pool_base_token_reserves: u64,
    pub pool_quote_token_reserves: u64,
    pub quote_amount_out: u64,
    pub lp_fee_basis_points: u64,
    pub lp_fee: u64,
    pub protocol_fee_basis_points: u64,
    pub protocol_fee: u64,
    pub quote_amount_out_without_lp_fee: u64,
    pub user_quote_amount_out: u64,
    pub pool: Pubkey,
    pub user: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub protocol_fee_recipient: Pubkey,
    pub protocol_fee_recipient_token_account: Pubkey,
    pub coin_creator: Pubkey,
    pub coin_creator_fee_basis_points: u64,
    pub coin_creator_fee: u64,
    /// Cashback fee basis points (PUMP_CASHBACK_README)
    pub cashback_fee_basis_points: u64,
    /// Cashback amount (PUMP_CASHBACK_README)
    pub cashback: u64,

    // === 额外的信息 ===
    #[borsh(skip)]
    pub is_pump_pool: bool,

    // === 额外账户信息 (from instruction accounts, not event data) ===
    #[borsh(skip)]
    pub base_mint: Pubkey,
    #[borsh(skip)]
    pub quote_mint: Pubkey,
    #[borsh(skip)]
    pub pool_base_token_account: Pubkey,
    #[borsh(skip)]
    pub pool_quote_token_account: Pubkey,
    #[borsh(skip)]
    pub coin_creator_vault_ata: Pubkey,
    #[borsh(skip)]
    pub coin_creator_vault_authority: Pubkey,
    #[borsh(skip)]
    pub base_token_program: Pubkey,
    #[borsh(skip)]
    pub quote_token_program: Pubkey,
}

/// PumpSwap Create Pool Event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PumpSwapCreatePoolEvent {
    pub metadata: EventMetadata,
    pub timestamp: i64,
    pub index: u16,
    pub creator: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_mint_decimals: u8,
    pub quote_mint_decimals: u8,
    pub base_amount_in: u64,
    pub quote_amount_in: u64,
    pub pool_base_amount: u64,
    pub pool_quote_amount: u64,
    pub minimum_liquidity: u64,
    pub initial_liquidity: u64,
    pub lp_token_amount_out: u64,
    pub pool_bump: u8,
    pub pool: Pubkey,
    pub lp_mint: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub coin_creator: Pubkey,
    /// IDL CreatePoolEvent 最后一列
    pub is_mayhem_mode: bool,
}

/// PumpSwap Pool Created Event - 指令解析版本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PumpSwapPoolCreated {
    pub metadata: EventMetadata,
    pub pool_account: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub token_a_vault: Pubkey,
    pub token_b_vault: Pubkey,
    pub lp_mint: Pubkey,
    pub creator: Pubkey,
    pub authority: Pubkey,
    pub initial_token_a_amount: u64,
    pub initial_token_b_amount: u64,
}

/// PumpSwap Trade Event - 指令解析版本
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct PumpSwapTrade {
//     pub metadata: EventMetadata,
//     pub pool_account: Pubkey,
//     pub user: Pubkey,
//     pub user_token_in_account: Pubkey,
//     pub user_token_out_account: Pubkey,
//     pub pool_token_in_vault: Pubkey,
//     pub pool_token_out_vault: Pubkey,
//     pub token_in_mint: Pubkey,
//     pub token_out_mint: Pubkey,
//     pub amount_in: u64,
//     pub minimum_amount_out: u64,
//     pub is_token_a_to_b: bool,
// }

/// PumpSwap Liquidity Added Event - Instruction parsing version
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PumpSwapLiquidityAdded {
    pub metadata: EventMetadata,
    pub timestamp: i64,
    pub lp_token_amount_out: u64,
    pub max_base_amount_in: u64,
    pub max_quote_amount_in: u64,
    pub user_base_token_reserves: u64,
    pub user_quote_token_reserves: u64,
    pub pool_base_token_reserves: u64,
    pub pool_quote_token_reserves: u64,
    pub base_amount_in: u64,
    pub quote_amount_in: u64,
    pub lp_mint_supply: u64,
    pub pool: Pubkey,
    pub user: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub user_pool_token_account: Pubkey,
}

/// PumpSwap Liquidity Removed Event - Instruction parsing version
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PumpSwapLiquidityRemoved {
    pub metadata: EventMetadata,
    pub timestamp: i64,
    pub lp_token_amount_in: u64,
    pub min_base_amount_out: u64,
    pub min_quote_amount_out: u64,
    pub user_base_token_reserves: u64,
    pub user_quote_token_reserves: u64,
    pub pool_base_token_reserves: u64,
    pub pool_quote_token_reserves: u64,
    pub base_amount_out: u64,
    pub quote_amount_out: u64,
    pub lp_mint_supply: u64,
    pub pool: Pubkey,
    pub user: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub user_pool_token_account: Pubkey,
}

/// PumpSwap Pool Updated Event - 指令解析版本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PumpSwapPoolUpdated {
    pub metadata: EventMetadata,
    pub pool_account: Pubkey,
    pub authority: Pubkey,
    pub admin: Pubkey,
    pub new_fee_rate: u64,
}

/// PumpSwap Fees Claimed Event - 指令解析版本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PumpSwapFeesClaimed {
    pub metadata: EventMetadata,
    pub pool_account: Pubkey,
    pub authority: Pubkey,
    pub admin: Pubkey,
    pub admin_token_a_account: Pubkey,
    pub admin_token_b_account: Pubkey,
    pub pool_fee_vault: Pubkey,
}

/// PumpSwap Deposit Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PumpSwapDepositEvent {
    pub metadata: EventMetadata,
    pub pool: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
}

/// PumpSwap Withdraw Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PumpSwapWithdrawEvent {
    pub metadata: EventMetadata,
    pub pool: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
}

/// Raydium CPMM Swap Event (基于IDL SwapEvent + swapBaseInput指令定义)
#[cfg_attr(feature = "parse-borsh", derive(BorshDeserialize))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumCpmmSwapEvent {
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub metadata: EventMetadata,

    // === Borsh 序列化字段（从 inner instruction 事件）===
    pub pool_id: Pubkey,
    pub input_amount: u64,
    pub output_amount: u64,

    // === 非 Borsh 字段 ===
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub input_vault_before: u64,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub output_vault_before: u64,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub input_transfer_fee: u64,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub output_transfer_fee: u64,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub base_input: bool,
    // === 指令参数字段 (暂时注释，以后可能会用到，AI不要删除) ===
    // pub amount_in: u64,
    // pub minimum_amount_out: u64,

    // === 指令账户字段 (暂时注释，以后可能会用到，AI不要删除) ===
    // pub payer: Pubkey,              // 0: payer
    // pub authority: Pubkey,          // 1: authority
    // pub amm_config: Pubkey,         // 2: ammConfig
    // pub pool_state: Pubkey,         // 3: poolState
    // pub input_token_account: Pubkey, // 4: inputTokenAccount
    // pub output_token_account: Pubkey, // 5: outputTokenAccount
    // pub input_vault: Pubkey,        // 6: inputVault
    // pub output_vault: Pubkey,       // 7: outputVault
    // pub input_token_mint: Pubkey,   // 10: inputTokenMint
    // pub output_token_mint: Pubkey,  // 11: outputTokenMint
}

/// Raydium CPMM Deposit Event
#[cfg_attr(feature = "parse-borsh", derive(BorshDeserialize))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumCpmmDepositEvent {
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub metadata: EventMetadata,

    // === Borsh 序列化字段（从 inner instruction 事件）===
    pub pool: Pubkey,
    pub token0_amount: u64,
    pub token1_amount: u64,
    pub lp_token_amount: u64,

    // === 非 Borsh 字段 ===
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub user: Pubkey,
}

/// Raydium CPMM Initialize Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumCpmmInitializeEvent {
    pub metadata: EventMetadata,
    pub pool: Pubkey,
    pub creator: Pubkey,
    pub init_amount0: u64,
    pub init_amount1: u64,
}

/// Raydium CPMM Withdraw Event
#[cfg_attr(feature = "parse-borsh", derive(BorshDeserialize))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumCpmmWithdrawEvent {
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub metadata: EventMetadata,

    // === Borsh 序列化字段（从 inner instruction 事件）===
    pub pool: Pubkey,
    pub lp_token_amount: u64,
    pub token0_amount: u64,
    pub token1_amount: u64,

    // === 非 Borsh 字段 ===
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub user: Pubkey,
}

/// Raydium CLMM Swap Event (基于IDL SwapEvent + swap指令定义)
#[cfg_attr(feature = "parse-borsh", derive(BorshDeserialize))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumClmmSwapEvent {
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub metadata: EventMetadata,

    // === IDL SwapEvent 事件字段 (Borsh 序列化字段) ===
    pub pool_state: Pubkey,
    pub token_account_0: Pubkey,
    pub token_account_1: Pubkey,
    pub amount_0: u64,
    pub amount_1: u64,
    pub zero_for_one: bool,
    pub sqrt_price_x64: u128,
    pub liquidity: u128,

    // === 非 Borsh 字段 ===
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub sender: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub transfer_fee_0: u64,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub transfer_fee_1: u64,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub tick: i32,
    // === 指令参数字段 (暂时注释，以后可能会用到，AI不要删除) ===
    // pub amount: u64,
    // pub other_amount_threshold: u64,
    // pub sqrt_price_limit_x64: u128,
    // pub is_base_input: bool,

    // === 指令账户字段 (暂时注释，以后可能会用到，AI不要删除) ===
    // TODO: 根据Raydium CLMM swap指令IDL添加账户字段
}

/// Raydium CLMM Close Position Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumClmmClosePositionEvent {
    pub metadata: EventMetadata,
    pub pool: Pubkey,
    pub user: Pubkey,
    pub position_nft_mint: Pubkey,
}

/// Raydium CLMM Decrease Liquidity Event
#[cfg_attr(feature = "parse-borsh", derive(BorshDeserialize))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumClmmDecreaseLiquidityEvent {
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub metadata: EventMetadata,

    // === Borsh 序列化字段 ===
    pub pool: Pubkey,
    pub position_nft_mint: Pubkey,
    pub amount0_min: u64,
    pub amount1_min: u64,
    pub liquidity: u128,

    // === 非 Borsh 字段 ===
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub user: Pubkey,
}

/// Raydium CLMM Collect Fee Event
#[cfg_attr(feature = "parse-borsh", derive(BorshDeserialize))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumClmmCollectFeeEvent {
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub metadata: EventMetadata,

    // === Borsh 序列化字段 ===
    pub pool_state: Pubkey,
    pub position_nft_mint: Pubkey,
    pub amount_0: u64,
    pub amount_1: u64,
}

/// Raydium CLMM Create Pool Event
#[cfg_attr(feature = "parse-borsh", derive(BorshDeserialize))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumClmmCreatePoolEvent {
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub metadata: EventMetadata,

    // === Borsh 序列化字段（从 inner instruction 事件）===
    pub pool: Pubkey,
    pub token_0_mint: Pubkey,
    pub token_1_mint: Pubkey,
    pub tick_spacing: u16,
    pub fee_rate: u32,
    pub sqrt_price_x64: u128,

    // === 非 Borsh 字段（从指令或账户） ===
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub creator: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub open_time: u64,
}

/// Raydium CLMM Increase Liquidity Event
#[cfg_attr(feature = "parse-borsh", derive(BorshDeserialize))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumClmmIncreaseLiquidityEvent {
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub metadata: EventMetadata,

    // === Borsh 序列化字段 ===
    pub pool: Pubkey,
    pub position_nft_mint: Pubkey,
    pub amount0_max: u64,
    pub amount1_max: u64,
    pub liquidity: u128,

    // === 非 Borsh 字段 ===
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub user: Pubkey,
}

/// Raydium CLMM Open Position with Token Extension NFT Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumClmmOpenPositionWithTokenExtNftEvent {
    pub metadata: EventMetadata,
    pub pool: Pubkey,
    pub user: Pubkey,
    pub position_nft_mint: Pubkey,
    pub tick_lower_index: i32,
    pub tick_upper_index: i32,
    pub liquidity: u128,
}

/// Raydium CLMM Open Position Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumClmmOpenPositionEvent {
    pub metadata: EventMetadata,
    pub pool: Pubkey,
    pub user: Pubkey,
    pub position_nft_mint: Pubkey,
    pub tick_lower_index: i32,
    pub tick_upper_index: i32,
    pub liquidity: u128,
}

/// Raydium AMM V4 Deposit Event (简化版)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumAmmDepositEvent {
    pub metadata: EventMetadata,
    pub amm_id: Pubkey,
    pub user: Pubkey,
    pub max_coin_amount: u64,
    pub max_pc_amount: u64,
}

/// Raydium AMM V4 Initialize Alt Event (简化版)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumAmmInitializeAltEvent {
    pub metadata: EventMetadata,
    pub amm_id: Pubkey,
    pub creator: Pubkey,
    pub nonce: u8,
    pub open_time: u64,
}

/// Raydium AMM V4 Withdraw Event (简化版)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumAmmWithdrawEvent {
    pub metadata: EventMetadata,
    pub amm_id: Pubkey,
    pub user: Pubkey,
    pub pool_coin_amount: u64,
}

/// Raydium AMM V4 Withdraw PnL Event (简化版)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumAmmWithdrawPnlEvent {
    pub metadata: EventMetadata,
    pub amm_id: Pubkey,
    pub user: Pubkey,
}

// ====================== Raydium AMM V4 Events ======================

/// Raydium AMM V4 Swap Event
#[cfg_attr(feature = "parse-borsh", derive(BorshDeserialize))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumAmmV4SwapEvent {
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub metadata: EventMetadata,

    // === Borsh 序列化字段（从 inner instruction 事件）===
    pub amm: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,

    // === 非 Borsh 字段 ===
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub minimum_amount_out: u64,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub max_amount_in: u64,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub token_program: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub amm_authority: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub amm_open_orders: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub amm_target_orders: Option<Pubkey>,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub pool_coin_token_account: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub pool_pc_token_account: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub serum_program: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub serum_market: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub serum_bids: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub serum_asks: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub serum_event_queue: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub serum_coin_vault_account: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub serum_pc_vault_account: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub serum_vault_signer: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub user_source_token_account: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub user_destination_token_account: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub user_source_owner: Pubkey,
}

/// Raydium AMM V4 Deposit Event
#[cfg_attr(feature = "parse-borsh", derive(BorshDeserialize))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumAmmV4DepositEvent {
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub metadata: EventMetadata,

    // === Borsh 序列化字段（从 inner instruction 事件）===
    pub amm: Pubkey,
    pub max_coin_amount: u64,
    pub max_pc_amount: u64,

    // === 非 Borsh 字段 ===
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub base_side: u64,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub token_program: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub amm_authority: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub amm_open_orders: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub amm_target_orders: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub lp_mint_address: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub pool_coin_token_account: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub pool_pc_token_account: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub serum_market: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub user_coin_token_account: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub user_pc_token_account: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub user_lp_token_account: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub user_owner: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub serum_event_queue: Pubkey,
}

/// Raydium AMM V4 Initialize2 Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumAmmV4Initialize2Event {
    pub metadata: EventMetadata,
    pub nonce: u8,
    pub open_time: u64,
    pub init_pc_amount: u64,
    pub init_coin_amount: u64,

    pub token_program: Pubkey,
    pub spl_associated_token_account: Pubkey,
    pub system_program: Pubkey,
    pub rent: Pubkey,
    pub amm: Pubkey,
    pub amm_authority: Pubkey,
    pub amm_open_orders: Pubkey,
    pub lp_mint: Pubkey,
    pub coin_mint: Pubkey,
    pub pc_mint: Pubkey,
    pub pool_coin_token_account: Pubkey,
    pub pool_pc_token_account: Pubkey,
    pub pool_withdraw_queue: Pubkey,
    pub amm_target_orders: Pubkey,
    pub pool_temp_lp: Pubkey,
    pub serum_program: Pubkey,
    pub serum_market: Pubkey,
    pub user_wallet: Pubkey,
    pub user_token_coin: Pubkey,
    pub user_token_pc: Pubkey,
    pub user_lp_token_account: Pubkey,
}

/// Raydium AMM V4 Withdraw Event
#[cfg_attr(feature = "parse-borsh", derive(BorshDeserialize))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumAmmV4WithdrawEvent {
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub metadata: EventMetadata,

    // === Borsh 序列化字段（从 inner instruction 事件）===
    pub amm: Pubkey,
    pub amount: u64,

    // === 非 Borsh 字段 ===
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub token_program: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub amm_authority: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub amm_open_orders: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub amm_target_orders: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub lp_mint_address: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub pool_coin_token_account: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub pool_pc_token_account: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub pool_withdraw_queue: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub pool_temp_lp_token_account: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub serum_program: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub serum_market: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub serum_coin_vault_account: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub serum_pc_vault_account: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub serum_vault_signer: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub user_lp_token_account: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub user_coin_token_account: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub user_pc_token_account: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub user_owner: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub serum_event_queue: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub serum_bids: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub serum_asks: Pubkey,
}

/// Raydium AMM V4 Withdraw PnL Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumAmmV4WithdrawPnlEvent {
    pub metadata: EventMetadata,

    pub token_program: Pubkey,
    pub amm: Pubkey,
    pub amm_config: Pubkey,
    pub amm_authority: Pubkey,
    pub amm_open_orders: Pubkey,
    pub pool_coin_token_account: Pubkey,
    pub pool_pc_token_account: Pubkey,
    pub coin_pnl_token_account: Pubkey,
    pub pc_pnl_token_account: Pubkey,
    pub pnl_owner: Pubkey,
    pub amm_target_orders: Pubkey,
    pub serum_program: Pubkey,
    pub serum_market: Pubkey,
    pub serum_event_queue: Pubkey,
    pub serum_coin_vault_account: Pubkey,
    pub serum_pc_vault_account: Pubkey,
    pub serum_vault_signer: Pubkey,
}

// ====================== Account Events ======================

/// Bonk (Raydium Launchpad) AmmCreatorFeeOn enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AmmCreatorFeeOn {
    QuoteToken = 0,
    BothToken = 1,
}

/// Bonk (Raydium Launchpad) VestingSchedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VestingSchedule {
    pub total_locked_amount: u64,
    pub cliff_period: u64,
    pub unlock_period: u64,
}

/// Bonk Pool State Account Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonkPoolStateAccountEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub pool_state: BonkPoolState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonkPoolState {
    pub epoch: u64,
    pub auth_bump: u8,
    pub status: u8,
    pub base_decimals: u8,
    pub quote_decimals: u8,
    pub migrate_type: u8,
    pub supply: u64,
    pub total_base_sell: u64,
    pub virtual_base: u64,
    pub virtual_quote: u64,
    pub real_base: u64,
    pub real_quote: u64,
    pub total_quote_fund_raising: u64,
    pub quote_protocol_fee: u64,
    pub platform_fee: u64,
    pub migrate_fee: u64,
    pub vesting_schedule: VestingSchedule,
    pub global_config: Pubkey,
    pub platform_config: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub creator: Pubkey,
    pub token_program_flag: u8,
    pub amm_creator_fee_on: AmmCreatorFeeOn,
    pub platform_vesting_share: u64,
    #[serde(with = "serde_big_array::BigArray")]
    pub padding: [u8; 54],
}

/// Bonk Global Config Account Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonkGlobalConfigAccountEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub global_config: BonkGlobalConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonkGlobalConfig {
    pub protocol_fee_rate: u64,
    pub trade_fee_rate: u64,
    pub migration_fee_rate: u64,
}

/// Bonk Platform Config Account Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonkPlatformConfigAccountEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub platform_config: BonkPlatformConfig,
}

/// Bonk (Raydium Launchpad) BondingCurveParam
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondingCurveParam {
    pub migrate_type: u8,
    pub migrate_cpmm_fee_on: u8,
    pub supply: u64,
    pub total_base_sell: u64,
    pub total_quote_fund_raising: u64,
    pub total_locked_amount: u64,
    pub cliff_period: u64,
    pub unlock_period: u64,
}

/// Bonk (Raydium Launchpad) PlatformCurveParam
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformCurveParam {
    pub epoch: u64,
    pub index: u8,
    pub global_config: Pubkey,
    pub bonding_curve_param: BondingCurveParam,
    #[serde(with = "serde_big_array::BigArray")]
    pub padding: [u64; 50],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonkPlatformConfig {
    pub epoch: u64,
    pub platform_fee_wallet: Pubkey,
    pub platform_nft_wallet: Pubkey,
    pub platform_scale: u64,
    pub creator_scale: u64,
    pub burn_scale: u64,
    pub fee_rate: u64,
    #[serde(with = "serde_big_array::BigArray")]
    pub name: [u8; 64],
    #[serde(with = "serde_big_array::BigArray")]
    pub web: [u8; 256],
    #[serde(with = "serde_big_array::BigArray")]
    pub img: [u8; 256],
    pub cpswap_config: Pubkey,
    pub creator_fee_rate: u64,
    pub transfer_fee_extension_auth: Pubkey,
    pub platform_vesting_wallet: Pubkey,
    pub platform_vesting_scale: u64,
    pub platform_cp_creator: Pubkey,
    #[serde(with = "serde_big_array::BigArray")]
    pub padding: [u8; 108],
    pub curve_params: Vec<PlatformCurveParam>,
}

/// PumpSwap Global Config Account Event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PumpSwapGlobalConfigAccountEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub global_config: PumpSwapGlobalConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PumpSwapGlobalConfig {
    pub admin: Pubkey,
    pub lp_fee_basis_points: u64,
    pub protocol_fee_basis_points: u64,
    pub disable_flags: u8,
    pub protocol_fee_recipients: [Pubkey; 8],
    pub coin_creator_fee_basis_points: u64,
    pub admin_set_coin_creator_authority: Pubkey,
    pub whitelist_pda: Pubkey,
    pub reserved_fee_recipient: Pubkey,
    pub mayhem_mode_enabled: bool,
    pub reserved_fee_recipients: [Pubkey; 7],
}

/// PumpSwap Pool Account Event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PumpSwapPoolAccountEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub pool: PumpSwapPool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PumpSwapPool {
    pub pool_bump: u8,
    pub index: u16,
    pub creator: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub lp_mint: Pubkey,
    pub pool_base_token_account: Pubkey,
    pub pool_quote_token_account: Pubkey,
    pub lp_supply: u64,
    pub coin_creator: Pubkey,
    pub is_mayhem_mode: bool,
    pub is_cashback_coin: bool,
}

/// PumpFun Bonding Curve Account Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PumpFunBondingCurveAccountEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub bonding_curve: PumpFunBondingCurve,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PumpFunBondingCurve {
    pub virtual_token_reserves: u64,
    pub virtual_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub token_total_supply: u64,
    pub complete: bool,
    /// Cashback 币种标记 (PUMP_CASHBACK_README)
    #[serde(default)]
    pub is_cashback_coin: bool,
}

/// PumpFun Global Account Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PumpFunGlobalAccountEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub global: PumpFunGlobal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PumpFunGlobal {
    pub initialized: bool,
    pub authority: Pubkey,
    pub fee_recipient: Pubkey,
    pub initial_virtual_token_reserves: u64,
    pub initial_virtual_sol_reserves: u64,
    pub initial_real_token_reserves: u64,
    pub token_total_supply: u64,
    pub fee_basis_points: u64,
    pub withdraw_authority: Pubkey,
    pub enable_migrate: bool,
    pub pool_migration_fee: u64,
    pub creator_fee_basis_points: u64,
    pub fee_recipients: [Pubkey; 8],
    pub set_creator_authority: Pubkey,
    pub admin_set_creator_authority: Pubkey,
    pub create_v2_enabled: bool,
    pub whitelist_pda: Pubkey,
    pub reserved_fee_recipient: Pubkey,
    pub mayhem_mode_enabled: bool,
    pub reserved_fee_recipients: [Pubkey; 7],
}

/// Raydium AMM V4 Info Account Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumAmmAmmInfoAccountEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub amm_info: RaydiumAmmInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumAmmInfo {
    pub status: u64,
    pub nonce: u64,
    pub order_num: u64,
    pub depth: u64,
    pub coin_decimals: u64,
    pub pc_decimals: u64,
    pub state: u64,
    pub reset_flag: u64,
    pub min_size: u64,
    pub vol_max_cut_ratio: u64,
    pub amount_wave_ratio: u64,
    pub coin_lot_size: u64,
    pub pc_lot_size: u64,
    pub min_price_multiplier: u64,
    pub max_price_multiplier: u64,
    pub sys_decimal_value: u64,
}

/// Raydium CLMM AMM Config Account Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumClmmAmmConfigAccountEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub amm_config: RaydiumClmmAmmConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumClmmAmmConfig {
    pub bump: u8,
    pub index: u16,
    pub owner: Pubkey,
    pub protocol_fee_rate: u32,
    pub trade_fee_rate: u32,
    pub tick_spacing: u16,
    pub fund_fee_rate: u32,
    pub fund_owner: Pubkey,
}

/// Raydium CLMM Pool State Account Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumClmmPoolStateAccountEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub pool_state: RaydiumClmmPoolState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumClmmPoolState {
    pub bump: [u8; 1],
    pub amm_config: Pubkey,
    pub owner: Pubkey,
    pub token_mint0: Pubkey,
    pub token_mint1: Pubkey,
    pub token_vault0: Pubkey,
    pub token_vault1: Pubkey,
    pub observation_key: Pubkey,
    pub mint_decimals0: u8,
    pub mint_decimals1: u8,
    pub tick_spacing: u16,
    pub liquidity: u128,
    pub sqrt_price_x64: u128,
    pub tick_current: i32,
}

/// Raydium CLMM Tick Array State Account Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumClmmTickArrayStateAccountEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub tick_array_state: RaydiumClmmTickArrayState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumClmmTickArrayState {
    pub discriminator: u64,
    pub pool_id: Pubkey,
    pub start_tick_index: i32,
    pub ticks: Vec<Tick>,
    pub initialized_tick_count: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tick {
    pub tick: i32,
    pub liquidity_net: i128,
    pub liquidity_gross: u128,
    pub fee_growth_outside_0_x64: u128,
    pub fee_growth_outside_1_x64: u128,
    pub reward_growths_outside_x64: [u128; 3],
}

/// Raydium CPMM AMM Config Account Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumCpmmAmmConfigAccountEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub amm_config: RaydiumCpmmAmmConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumCpmmAmmConfig {
    pub bump: u8,
    pub disable_create_pool: bool,
    pub index: u16,
    pub trade_fee_rate: u64,
    pub protocol_fee_rate: u64,
    pub fund_fee_rate: u64,
    pub create_pool_fee: u64,
    pub protocol_owner: Pubkey,
    pub fund_owner: Pubkey,
    pub creator_fee_rate: u64,
    pub padding: [u64; 15],
}

/// Raydium CPMM Pool State Account Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumCpmmPoolStateAccountEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub pool_state: RaydiumCpmmPoolState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumCpmmPoolState {
    pub amm_config: Pubkey,
    pub pool_creator: Pubkey,
    pub token_0_vault: Pubkey,
    pub token_1_vault: Pubkey,
    pub lp_mint: Pubkey,
    pub token_0_mint: Pubkey,
    pub token_1_mint: Pubkey,
    pub token_0_program: Pubkey,
    pub token_1_program: Pubkey,
    pub observation_key: Pubkey,
    pub auth_bump: u8,
    pub status: u8,
    pub lp_mint_decimals: u8,
    pub mint_0_decimals: u8,
    pub mint_1_decimals: u8,
    pub lp_supply: u64,
    pub protocol_fees_token_0: u64,
    pub protocol_fees_token_1: u64,
    pub fund_fees_token_0: u64,
    pub fund_fees_token_1: u64,
    pub open_time: u64,
    pub recent_epoch: u64,
    pub creator_fee_on: u8,
    pub enable_creator_fee: bool,
    pub padding1: [u8; 6],
    pub creator_fees_token_0: u64,
    pub creator_fees_token_1: u64,
    pub padding: [u64; 28],
}

/// Token Info Event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenInfoEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub supply: u64,
    pub decimals: u8,
}

/// Token Account Event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenAccountEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    #[serde(default)]
    pub mint: Pubkey,
    pub amount: Option<u64>,
    pub token_owner: Pubkey,
}

/// Nonce Account Event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NonceAccountEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub nonce: String,
    pub authority: String,
}

// ====================== Orca Whirlpool Events ======================

/// Orca Whirlpool Swap Event (基于 TradedEvent，不是 SwapEvent)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "parse-borsh", derive(BorshDeserialize))]
pub struct OrcaWhirlpoolSwapEvent {
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub metadata: EventMetadata,

    // === Borsh 序列化字段（从 inner instruction data 读取）===
    pub whirlpool: Pubkey,  // 32 bytes
    pub input_amount: u64,  // 8 bytes
    pub output_amount: u64, // 8 bytes
    pub a_to_b: bool,       // 1 byte

    // === 非 Borsh 字段（从日志或其他来源填充）===
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub pre_sqrt_price: u128,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub post_sqrt_price: u128,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub input_transfer_fee: u64,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub output_transfer_fee: u64,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub lp_fee: u64,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub protocol_fee: u64,
    // === 指令参数字段 (暂时注释，以后可能会用到，AI不要删除) ===
    // pub amount: u64,
    // pub other_amount_threshold: u64,
    // pub sqrt_price_limit: u128,
    // pub amount_specified_is_input: bool,

    // === 指令账户字段 (暂时注释，以后可能会用到，AI不要删除) ===
    // pub token_authority: Pubkey,    // 1: tokenAuthority
    // pub token_owner_account_a: Pubkey, // 3: tokenOwnerAccountA
    // pub token_vault_a: Pubkey,      // 4: tokenVaultA
    // pub token_owner_account_b: Pubkey, // 5: tokenOwnerAccountB
    // pub token_vault_b: Pubkey,      // 6: tokenVaultB
    // pub tick_array_0: Pubkey,       // 7: tickArray0
    // pub tick_array_1: Pubkey,       // 8: tickArray1
    // pub tick_array_2: Pubkey,       // 9: tickArray2
}

/// Orca Whirlpool Liquidity Increased Event
#[cfg_attr(feature = "parse-borsh", derive(BorshDeserialize))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrcaWhirlpoolLiquidityIncreasedEvent {
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub metadata: EventMetadata,

    // === Borsh 序列化字段（从 inner instruction data 读取）===
    pub whirlpool: Pubkey,   // 32 bytes
    pub liquidity: u128,     // 16 bytes
    pub token_a_amount: u64, // 8 bytes
    pub token_b_amount: u64, // 8 bytes

    // === 非 Borsh 字段（从日志或其他来源填充）===
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub position: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub tick_lower_index: i32,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub tick_upper_index: i32,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub token_a_transfer_fee: u64,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub token_b_transfer_fee: u64,
}

/// Orca Whirlpool Liquidity Decreased Event
#[cfg_attr(feature = "parse-borsh", derive(BorshDeserialize))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrcaWhirlpoolLiquidityDecreasedEvent {
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub metadata: EventMetadata,

    // === Borsh 序列化字段（从 inner instruction data 读取）===
    pub whirlpool: Pubkey,   // 32 bytes
    pub liquidity: u128,     // 16 bytes
    pub token_a_amount: u64, // 8 bytes
    pub token_b_amount: u64, // 8 bytes

    // === 非 Borsh 字段（从日志或其他来源填充）===
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub position: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub tick_lower_index: i32,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub tick_upper_index: i32,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub token_a_transfer_fee: u64,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub token_b_transfer_fee: u64,
}

/// Orca Whirlpool Pool Initialized Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrcaWhirlpoolPoolInitializedEvent {
    pub metadata: EventMetadata,
    pub whirlpool: Pubkey,
    pub whirlpools_config: Pubkey,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub tick_spacing: u16,
    pub token_program_a: Pubkey,
    pub token_program_b: Pubkey,
    pub decimals_a: u8,
    pub decimals_b: u8,
    pub initial_sqrt_price: u128,
}

// ====================== Meteora Pools Events ======================

/// Meteora Pools Swap Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteoraPoolsSwapEvent {
    pub metadata: EventMetadata,
    pub in_amount: u64,
    pub out_amount: u64,
    pub trade_fee: u64,
    pub admin_fee: u64, // IDL字段名: adminFee
    pub host_fee: u64,
}

/// Meteora Pools Add Liquidity Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteoraPoolsAddLiquidityEvent {
    pub metadata: EventMetadata,
    pub lp_mint_amount: u64,
    pub token_a_amount: u64,
    pub token_b_amount: u64,
}

/// Meteora Pools Remove Liquidity Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteoraPoolsRemoveLiquidityEvent {
    pub metadata: EventMetadata,
    pub lp_unmint_amount: u64,
    pub token_a_out_amount: u64,
    pub token_b_out_amount: u64,
}

/// Meteora Pools Bootstrap Liquidity Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteoraPoolsBootstrapLiquidityEvent {
    pub metadata: EventMetadata,
    pub lp_mint_amount: u64,
    pub token_a_amount: u64,
    pub token_b_amount: u64,
    pub pool: Pubkey,
}

/// Meteora Pools Pool Created Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteoraPoolsPoolCreatedEvent {
    pub metadata: EventMetadata,
    pub lp_mint: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub pool_type: u8,
    pub pool: Pubkey,
}

/// Meteora Pools Set Pool Fees Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteoraPoolsSetPoolFeesEvent {
    pub metadata: EventMetadata,
    pub trade_fee_numerator: u64,
    pub trade_fee_denominator: u64,
    pub owner_trade_fee_numerator: u64, // IDL字段名: ownerTradeFeeNumerator
    pub owner_trade_fee_denominator: u64, // IDL字段名: ownerTradeFeeDenominator
    pub pool: Pubkey,
}

// ====================== Meteora DAMM V2 Events ======================

/// Meteora DAMM V2 Swap Event
#[cfg_attr(feature = "parse-borsh", derive(BorshDeserialize))]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MeteoraDammV2SwapEvent {
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub metadata: EventMetadata,

    // === Borsh 序列化字段（从 inner instruction data 读取）===
    pub pool: Pubkey,       // 32 bytes
    pub amount_in: u64,     // 8 bytes
    pub output_amount: u64, // 8 bytes

    // === 非 Borsh 字段（从日志或其他来源填充）===
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub trade_direction: u8,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub has_referral: bool,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub minimum_amount_out: u64,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub next_sqrt_price: u128,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub lp_fee: u64,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub protocol_fee: u64,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub partner_fee: u64,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub referral_fee: u64,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub actual_amount_in: u64,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub current_timestamp: u64,
    // ---------- 账号 -------------
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub token_a_vault: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub token_b_vault: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub token_a_mint: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub token_b_mint: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub token_a_program: Pubkey,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub token_b_program: Pubkey,
}

/// Meteora DAMM V2 Add Liquidity Event
#[cfg_attr(feature = "parse-borsh", derive(BorshDeserialize))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteoraDammV2AddLiquidityEvent {
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub metadata: EventMetadata,

    // === Borsh 序列化字段（从 inner instruction data 读取）===
    pub pool: Pubkey,        // 32 bytes
    pub position: Pubkey,    // 32 bytes
    pub owner: Pubkey,       // 32 bytes
    pub token_a_amount: u64, // 8 bytes
    pub token_b_amount: u64, // 8 bytes

    // === 非 Borsh 字段（从日志填充）===
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub liquidity_delta: u128,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub token_a_amount_threshold: u64,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub token_b_amount_threshold: u64,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub total_amount_a: u64,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub total_amount_b: u64,
}

/// Meteora DAMM V2 Remove Liquidity Event
#[cfg_attr(feature = "parse-borsh", derive(BorshDeserialize))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteoraDammV2RemoveLiquidityEvent {
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub metadata: EventMetadata,

    // === Borsh 序列化字段（从 inner instruction data 读取）===
    pub pool: Pubkey,        // 32 bytes
    pub position: Pubkey,    // 32 bytes
    pub owner: Pubkey,       // 32 bytes
    pub token_a_amount: u64, // 8 bytes
    pub token_b_amount: u64, // 8 bytes

    // === 非 Borsh 字段（从日志填充）===
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub liquidity_delta: u128,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub token_a_amount_threshold: u64,
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub token_b_amount_threshold: u64,
}

/// Meteora DAMM V2 Create Position Event
#[cfg_attr(feature = "parse-borsh", derive(BorshDeserialize))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteoraDammV2CreatePositionEvent {
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub metadata: EventMetadata,

    // === Borsh 序列化字段（从 inner instruction data 读取）===
    pub pool: Pubkey,              // 32 bytes
    pub owner: Pubkey,             // 32 bytes
    pub position: Pubkey,          // 32 bytes
    pub position_nft_mint: Pubkey, // 32 bytes
}

/// Meteora DAMM V2 Close Position Event
#[cfg_attr(feature = "parse-borsh", derive(BorshDeserialize))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteoraDammV2ClosePositionEvent {
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub metadata: EventMetadata,

    // === Borsh 序列化字段（从 inner instruction data 读取）===
    pub pool: Pubkey,              // 32 bytes
    pub owner: Pubkey,             // 32 bytes
    pub position: Pubkey,          // 32 bytes
    pub position_nft_mint: Pubkey, // 32 bytes
}

/// Meteora DLMM Swap Event
#[cfg_attr(feature = "parse-borsh", derive(BorshDeserialize))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteoraDlmmSwapEvent {
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub metadata: EventMetadata,

    // === Borsh 序列化字段（从 inner instruction data 读取）===
    pub pool: Pubkey,      // 32 bytes
    pub from: Pubkey,      // 32 bytes
    pub start_bin_id: i32, // 4 bytes
    pub end_bin_id: i32,   // 4 bytes
    pub amount_in: u64,    // 8 bytes
    pub amount_out: u64,   // 8 bytes
    pub swap_for_y: bool,  // 1 byte
    pub fee: u64,          // 8 bytes
    pub protocol_fee: u64, // 8 bytes
    pub fee_bps: u128,     // 16 bytes
    pub host_fee: u64,     // 8 bytes
}

/// Meteora DLMM Add Liquidity Event
#[cfg_attr(feature = "parse-borsh", derive(BorshDeserialize))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteoraDlmmAddLiquidityEvent {
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub metadata: EventMetadata,

    // === Borsh 序列化字段（从 inner instruction data 读取）===
    pub pool: Pubkey,       // 32 bytes
    pub from: Pubkey,       // 32 bytes
    pub position: Pubkey,   // 32 bytes
    pub amounts: [u64; 2],  // 16 bytes (2 * 8)
    pub active_bin_id: i32, // 4 bytes
}

/// Meteora DLMM Remove Liquidity Event
#[cfg_attr(feature = "parse-borsh", derive(BorshDeserialize))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteoraDlmmRemoveLiquidityEvent {
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub metadata: EventMetadata,

    // === Borsh 序列化字段（从 inner instruction data 读取）===
    pub pool: Pubkey,       // 32 bytes
    pub from: Pubkey,       // 32 bytes
    pub position: Pubkey,   // 32 bytes
    pub amounts: [u64; 2],  // 16 bytes (2 * 8)
    pub active_bin_id: i32, // 4 bytes
}

/// Meteora DLMM Initialize Pool Event
#[cfg_attr(feature = "parse-borsh", derive(BorshDeserialize))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteoraDlmmInitializePoolEvent {
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub metadata: EventMetadata,

    // === Borsh 序列化字段（从 inner instruction data 读取）===
    pub pool: Pubkey,       // 32 bytes
    pub creator: Pubkey,    // 32 bytes
    pub active_bin_id: i32, // 4 bytes
    pub bin_step: u16,      // 2 bytes
}

/// Meteora DLMM Initialize Bin Array Event
#[cfg_attr(feature = "parse-borsh", derive(BorshDeserialize))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteoraDlmmInitializeBinArrayEvent {
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub metadata: EventMetadata,

    // === Borsh 序列化字段（从 inner instruction data 读取）===
    pub pool: Pubkey,      // 32 bytes
    pub bin_array: Pubkey, // 32 bytes
    pub index: i64,        // 8 bytes
}

/// Meteora DLMM Create Position Event
#[cfg_attr(feature = "parse-borsh", derive(BorshDeserialize))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteoraDlmmCreatePositionEvent {
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub metadata: EventMetadata,

    // === Borsh 序列化字段（从 inner instruction data 读取）===
    pub pool: Pubkey,      // 32 bytes
    pub position: Pubkey,  // 32 bytes
    pub owner: Pubkey,     // 32 bytes
    pub lower_bin_id: i32, // 4 bytes
    pub width: u32,        // 4 bytes
}

/// Meteora DLMM Close Position Event
#[cfg_attr(feature = "parse-borsh", derive(BorshDeserialize))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteoraDlmmClosePositionEvent {
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub metadata: EventMetadata,

    // === Borsh 序列化字段（从 inner instruction data 读取）===
    pub pool: Pubkey,     // 32 bytes
    pub position: Pubkey, // 32 bytes
    pub owner: Pubkey,    // 32 bytes
}

/// Meteora DLMM Claim Fee Event
#[cfg_attr(feature = "parse-borsh", derive(BorshDeserialize))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteoraDlmmClaimFeeEvent {
    #[cfg_attr(feature = "parse-borsh", borsh(skip))]
    pub metadata: EventMetadata,

    // === Borsh 序列化字段（从 inner instruction data 读取）===
    pub pool: Pubkey,     // 32 bytes
    pub position: Pubkey, // 32 bytes
    pub owner: Pubkey,    // 32 bytes
    pub fee_x: u64,       // 8 bytes
    pub fee_y: u64,       // 8 bytes
}

// ====================== 统一的 DEX 事件枚举 ======================

/// 统一的 DEX 事件枚举 - 参考 sol-dex-shreds 的做法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DexEvent {
    // PumpFun 事件
    PumpFunCreate(PumpFunCreateTokenEvent),     // - 已对接
    PumpFunCreateV2(PumpFunCreateV2TokenEvent), // - 已对接 (CreateV2 / Mayhem)
    PumpFunTrade(PumpFunTradeEvent),            // - 已对接 (统一交易事件，包含所有交易类型)
    PumpFunBuy(PumpFunTradeEvent),              // - 已对接 (仅买入事件，用于过滤)
    PumpFunSell(PumpFunTradeEvent),             // - 已对接 (仅卖出事件，用于过滤)
    PumpFunBuyExactSolIn(PumpFunTradeEvent),    // - 已对接 (精确SOL买入事件，用于过滤)
    PumpFunMigrate(PumpFunMigrateEvent),        // - 已对接

    // PumpSwap 事件
    PumpSwapTrade(PumpSwapTradeEvent), // - 已对接 (buy/sell/buy_exact_sol_in)
    PumpSwapBuy(PumpSwapBuyEvent),     // - 已对接 (legacy)
    PumpSwapSell(PumpSwapSellEvent),   // - 已对接 (legacy)
    PumpSwapCreatePool(PumpSwapCreatePoolEvent), // - 已对接
    PumpSwapLiquidityAdded(PumpSwapLiquidityAdded), // - 已对接
    PumpSwapLiquidityRemoved(PumpSwapLiquidityRemoved), // - 已对接

    // Meteora DAMM V2 事件
    MeteoraDammV2Swap(MeteoraDammV2SwapEvent), // - 已对接
    MeteoraDammV2CreatePosition(MeteoraDammV2CreatePositionEvent), // - 已对接
    MeteoraDammV2ClosePosition(MeteoraDammV2ClosePositionEvent), // - 已对接
    MeteoraDammV2AddLiquidity(MeteoraDammV2AddLiquidityEvent), // - 已对接
    MeteoraDammV2RemoveLiquidity(MeteoraDammV2RemoveLiquidityEvent), // - 已对接

    // Bonk 事件
    BonkTrade(BonkTradeEvent),
    BonkPoolCreate(BonkPoolCreateEvent),
    BonkMigrateAmm(BonkMigrateAmmEvent),

    // Raydium CLMM 事件
    RaydiumClmmSwap(RaydiumClmmSwapEvent),
    RaydiumClmmCreatePool(RaydiumClmmCreatePoolEvent),
    RaydiumClmmOpenPosition(RaydiumClmmOpenPositionEvent),
    RaydiumClmmOpenPositionWithTokenExtNft(RaydiumClmmOpenPositionWithTokenExtNftEvent),
    RaydiumClmmClosePosition(RaydiumClmmClosePositionEvent),
    RaydiumClmmIncreaseLiquidity(RaydiumClmmIncreaseLiquidityEvent),
    RaydiumClmmDecreaseLiquidity(RaydiumClmmDecreaseLiquidityEvent),
    RaydiumClmmCollectFee(RaydiumClmmCollectFeeEvent),

    // Raydium CPMM 事件
    RaydiumCpmmSwap(RaydiumCpmmSwapEvent),
    RaydiumCpmmDeposit(RaydiumCpmmDepositEvent),
    RaydiumCpmmWithdraw(RaydiumCpmmWithdrawEvent),
    RaydiumCpmmInitialize(RaydiumCpmmInitializeEvent),

    // Raydium AMM V4 事件
    RaydiumAmmV4Swap(RaydiumAmmV4SwapEvent),
    RaydiumAmmV4Deposit(RaydiumAmmV4DepositEvent),
    RaydiumAmmV4Initialize2(RaydiumAmmV4Initialize2Event),
    RaydiumAmmV4Withdraw(RaydiumAmmV4WithdrawEvent),
    RaydiumAmmV4WithdrawPnl(RaydiumAmmV4WithdrawPnlEvent),

    // Orca Whirlpool 事件
    OrcaWhirlpoolSwap(OrcaWhirlpoolSwapEvent),
    OrcaWhirlpoolLiquidityIncreased(OrcaWhirlpoolLiquidityIncreasedEvent),
    OrcaWhirlpoolLiquidityDecreased(OrcaWhirlpoolLiquidityDecreasedEvent),
    OrcaWhirlpoolPoolInitialized(OrcaWhirlpoolPoolInitializedEvent),

    // Meteora Pools 事件
    MeteoraPoolsSwap(MeteoraPoolsSwapEvent),
    MeteoraPoolsAddLiquidity(MeteoraPoolsAddLiquidityEvent),
    MeteoraPoolsRemoveLiquidity(MeteoraPoolsRemoveLiquidityEvent),
    MeteoraPoolsBootstrapLiquidity(MeteoraPoolsBootstrapLiquidityEvent),
    MeteoraPoolsPoolCreated(MeteoraPoolsPoolCreatedEvent),
    MeteoraPoolsSetPoolFees(MeteoraPoolsSetPoolFeesEvent),

    // Meteora DLMM 事件
    MeteoraDlmmSwap(MeteoraDlmmSwapEvent),
    MeteoraDlmmAddLiquidity(MeteoraDlmmAddLiquidityEvent),
    MeteoraDlmmRemoveLiquidity(MeteoraDlmmRemoveLiquidityEvent),
    MeteoraDlmmInitializePool(MeteoraDlmmInitializePoolEvent),
    MeteoraDlmmInitializeBinArray(MeteoraDlmmInitializeBinArrayEvent),
    MeteoraDlmmCreatePosition(MeteoraDlmmCreatePositionEvent),
    MeteoraDlmmClosePosition(MeteoraDlmmClosePositionEvent),
    MeteoraDlmmClaimFee(MeteoraDlmmClaimFeeEvent),

    // 账户事件
    TokenInfo(TokenInfoEvent),       // - 已对接
    TokenAccount(TokenAccountEvent), // - 已对接
    NonceAccount(NonceAccountEvent), // - 已对接
    PumpSwapGlobalConfigAccount(PumpSwapGlobalConfigAccountEvent), // - 已对接
    PumpSwapPoolAccount(PumpSwapPoolAccountEvent), // - 已对接

    // 区块元数据事件
    BlockMeta(BlockMetaEvent),

    // 错误事件
    Error(String),
}

// 静态默认 EventMetadata，用于 Error 事件
use once_cell::sync::Lazy;
static DEFAULT_METADATA: Lazy<EventMetadata> = Lazy::new(|| EventMetadata {
    signature: Signature::from([0u8; 64]),
    slot: 0,
    tx_index: 0,
    block_time_us: 0,
    grpc_recv_us: 0,
    recent_blockhash: None,
});

impl DexEvent {
    /// 获取事件的元数据
    pub fn metadata(&self) -> &EventMetadata {
        match self {
            // PumpFun 事件
            DexEvent::PumpFunCreate(e) => &e.metadata,
            DexEvent::PumpFunCreateV2(e) => &e.metadata,
            DexEvent::PumpFunTrade(e) => &e.metadata,
            DexEvent::PumpFunBuy(e) => &e.metadata,
            DexEvent::PumpFunSell(e) => &e.metadata,
            DexEvent::PumpFunBuyExactSolIn(e) => &e.metadata,
            DexEvent::PumpFunMigrate(e) => &e.metadata,

            // PumpSwap 事件
            DexEvent::PumpSwapTrade(e) => &e.metadata,
            DexEvent::PumpSwapBuy(e) => &e.metadata,
            DexEvent::PumpSwapSell(e) => &e.metadata,
            DexEvent::PumpSwapCreatePool(e) => &e.metadata,
            DexEvent::PumpSwapLiquidityAdded(e) => &e.metadata,
            DexEvent::PumpSwapLiquidityRemoved(e) => &e.metadata,

            // Meteora DAMM V2 事件
            DexEvent::MeteoraDammV2Swap(e) => &e.metadata,
            DexEvent::MeteoraDammV2CreatePosition(e) => &e.metadata,
            DexEvent::MeteoraDammV2ClosePosition(e) => &e.metadata,
            DexEvent::MeteoraDammV2AddLiquidity(e) => &e.metadata,
            DexEvent::MeteoraDammV2RemoveLiquidity(e) => &e.metadata,

            // Bonk 事件
            DexEvent::BonkTrade(e) => &e.metadata,
            DexEvent::BonkPoolCreate(e) => &e.metadata,
            DexEvent::BonkMigrateAmm(e) => &e.metadata,

            // Raydium CLMM 事件
            DexEvent::RaydiumClmmSwap(e) => &e.metadata,
            DexEvent::RaydiumClmmCreatePool(e) => &e.metadata,
            DexEvent::RaydiumClmmOpenPosition(e) => &e.metadata,
            DexEvent::RaydiumClmmOpenPositionWithTokenExtNft(e) => &e.metadata,
            DexEvent::RaydiumClmmClosePosition(e) => &e.metadata,
            DexEvent::RaydiumClmmIncreaseLiquidity(e) => &e.metadata,
            DexEvent::RaydiumClmmDecreaseLiquidity(e) => &e.metadata,
            DexEvent::RaydiumClmmCollectFee(e) => &e.metadata,

            // Raydium CPMM 事件
            DexEvent::RaydiumCpmmSwap(e) => &e.metadata,
            DexEvent::RaydiumCpmmDeposit(e) => &e.metadata,
            DexEvent::RaydiumCpmmWithdraw(e) => &e.metadata,
            DexEvent::RaydiumCpmmInitialize(e) => &e.metadata,

            // Raydium AMM V4 事件
            DexEvent::RaydiumAmmV4Swap(e) => &e.metadata,
            DexEvent::RaydiumAmmV4Deposit(e) => &e.metadata,
            DexEvent::RaydiumAmmV4Initialize2(e) => &e.metadata,
            DexEvent::RaydiumAmmV4Withdraw(e) => &e.metadata,
            DexEvent::RaydiumAmmV4WithdrawPnl(e) => &e.metadata,

            // Orca Whirlpool 事件
            DexEvent::OrcaWhirlpoolSwap(e) => &e.metadata,
            DexEvent::OrcaWhirlpoolLiquidityIncreased(e) => &e.metadata,
            DexEvent::OrcaWhirlpoolLiquidityDecreased(e) => &e.metadata,
            DexEvent::OrcaWhirlpoolPoolInitialized(e) => &e.metadata,

            // Meteora Pools 事件
            DexEvent::MeteoraPoolsSwap(e) => &e.metadata,
            DexEvent::MeteoraPoolsAddLiquidity(e) => &e.metadata,
            DexEvent::MeteoraPoolsRemoveLiquidity(e) => &e.metadata,
            DexEvent::MeteoraPoolsBootstrapLiquidity(e) => &e.metadata,
            DexEvent::MeteoraPoolsPoolCreated(e) => &e.metadata,
            DexEvent::MeteoraPoolsSetPoolFees(e) => &e.metadata,

            // Meteora DLMM 事件
            DexEvent::MeteoraDlmmSwap(e) => &e.metadata,
            DexEvent::MeteoraDlmmAddLiquidity(e) => &e.metadata,
            DexEvent::MeteoraDlmmRemoveLiquidity(e) => &e.metadata,
            DexEvent::MeteoraDlmmInitializePool(e) => &e.metadata,
            DexEvent::MeteoraDlmmInitializeBinArray(e) => &e.metadata,
            DexEvent::MeteoraDlmmCreatePosition(e) => &e.metadata,
            DexEvent::MeteoraDlmmClosePosition(e) => &e.metadata,
            DexEvent::MeteoraDlmmClaimFee(e) => &e.metadata,

            // 账户事件
            DexEvent::TokenInfo(e) => &e.metadata,
            DexEvent::TokenAccount(e) => &e.metadata,
            DexEvent::NonceAccount(e) => &e.metadata,
            DexEvent::PumpSwapGlobalConfigAccount(e) => &e.metadata,
            DexEvent::PumpSwapPoolAccount(e) => &e.metadata,

            // 区块元数据事件
            DexEvent::BlockMeta(e) => &e.metadata,

            // 错误事件 - 返回默认元数据
            DexEvent::Error(_) => &DEFAULT_METADATA,
        }
    }

    /// Mutable metadata for filling shared fields (e.g. recent_blockhash). Returns None for Error variant.
    pub fn metadata_mut(&mut self) -> Option<&mut EventMetadata> {
        match self {
            DexEvent::PumpFunCreate(e) => Some(&mut e.metadata),
            DexEvent::PumpFunCreateV2(e) => Some(&mut e.metadata),
            DexEvent::PumpFunTrade(e) => Some(&mut e.metadata),
            DexEvent::PumpFunBuy(e) => Some(&mut e.metadata),
            DexEvent::PumpFunSell(e) => Some(&mut e.metadata),
            DexEvent::PumpFunBuyExactSolIn(e) => Some(&mut e.metadata),
            DexEvent::PumpFunMigrate(e) => Some(&mut e.metadata),
            DexEvent::PumpSwapTrade(e) => Some(&mut e.metadata),
            DexEvent::PumpSwapBuy(e) => Some(&mut e.metadata),
            DexEvent::PumpSwapSell(e) => Some(&mut e.metadata),
            DexEvent::PumpSwapCreatePool(e) => Some(&mut e.metadata),
            DexEvent::PumpSwapLiquidityAdded(e) => Some(&mut e.metadata),
            DexEvent::PumpSwapLiquidityRemoved(e) => Some(&mut e.metadata),
            DexEvent::MeteoraDammV2Swap(e) => Some(&mut e.metadata),
            DexEvent::MeteoraDammV2CreatePosition(e) => Some(&mut e.metadata),
            DexEvent::MeteoraDammV2ClosePosition(e) => Some(&mut e.metadata),
            DexEvent::MeteoraDammV2AddLiquidity(e) => Some(&mut e.metadata),
            DexEvent::MeteoraDammV2RemoveLiquidity(e) => Some(&mut e.metadata),
            DexEvent::BonkTrade(e) => Some(&mut e.metadata),
            DexEvent::BonkPoolCreate(e) => Some(&mut e.metadata),
            DexEvent::BonkMigrateAmm(e) => Some(&mut e.metadata),
            DexEvent::RaydiumClmmSwap(e) => Some(&mut e.metadata),
            DexEvent::RaydiumClmmCreatePool(e) => Some(&mut e.metadata),
            DexEvent::RaydiumClmmOpenPosition(e) => Some(&mut e.metadata),
            DexEvent::RaydiumClmmOpenPositionWithTokenExtNft(e) => Some(&mut e.metadata),
            DexEvent::RaydiumClmmClosePosition(e) => Some(&mut e.metadata),
            DexEvent::RaydiumClmmIncreaseLiquidity(e) => Some(&mut e.metadata),
            DexEvent::RaydiumClmmDecreaseLiquidity(e) => Some(&mut e.metadata),
            DexEvent::RaydiumClmmCollectFee(e) => Some(&mut e.metadata),
            DexEvent::RaydiumCpmmSwap(e) => Some(&mut e.metadata),
            DexEvent::RaydiumCpmmDeposit(e) => Some(&mut e.metadata),
            DexEvent::RaydiumCpmmWithdraw(e) => Some(&mut e.metadata),
            DexEvent::RaydiumCpmmInitialize(e) => Some(&mut e.metadata),
            DexEvent::RaydiumAmmV4Swap(e) => Some(&mut e.metadata),
            DexEvent::RaydiumAmmV4Deposit(e) => Some(&mut e.metadata),
            DexEvent::RaydiumAmmV4Initialize2(e) => Some(&mut e.metadata),
            DexEvent::RaydiumAmmV4Withdraw(e) => Some(&mut e.metadata),
            DexEvent::RaydiumAmmV4WithdrawPnl(e) => Some(&mut e.metadata),
            DexEvent::OrcaWhirlpoolSwap(e) => Some(&mut e.metadata),
            DexEvent::OrcaWhirlpoolLiquidityIncreased(e) => Some(&mut e.metadata),
            DexEvent::OrcaWhirlpoolLiquidityDecreased(e) => Some(&mut e.metadata),
            DexEvent::OrcaWhirlpoolPoolInitialized(e) => Some(&mut e.metadata),
            DexEvent::MeteoraPoolsSwap(e) => Some(&mut e.metadata),
            DexEvent::MeteoraPoolsAddLiquidity(e) => Some(&mut e.metadata),
            DexEvent::MeteoraPoolsRemoveLiquidity(e) => Some(&mut e.metadata),
            DexEvent::MeteoraPoolsBootstrapLiquidity(e) => Some(&mut e.metadata),
            DexEvent::MeteoraPoolsPoolCreated(e) => Some(&mut e.metadata),
            DexEvent::MeteoraPoolsSetPoolFees(e) => Some(&mut e.metadata),
            DexEvent::MeteoraDlmmSwap(e) => Some(&mut e.metadata),
            DexEvent::MeteoraDlmmAddLiquidity(e) => Some(&mut e.metadata),
            DexEvent::MeteoraDlmmRemoveLiquidity(e) => Some(&mut e.metadata),
            DexEvent::MeteoraDlmmInitializePool(e) => Some(&mut e.metadata),
            DexEvent::MeteoraDlmmInitializeBinArray(e) => Some(&mut e.metadata),
            DexEvent::MeteoraDlmmCreatePosition(e) => Some(&mut e.metadata),
            DexEvent::MeteoraDlmmClosePosition(e) => Some(&mut e.metadata),
            DexEvent::MeteoraDlmmClaimFee(e) => Some(&mut e.metadata),
            DexEvent::TokenInfo(e) => Some(&mut e.metadata),
            DexEvent::TokenAccount(e) => Some(&mut e.metadata),
            DexEvent::NonceAccount(e) => Some(&mut e.metadata),
            DexEvent::PumpSwapGlobalConfigAccount(e) => Some(&mut e.metadata),
            DexEvent::PumpSwapPoolAccount(e) => Some(&mut e.metadata),
            DexEvent::BlockMeta(e) => Some(&mut e.metadata),
            DexEvent::Error(_) => None,
        }
    }
}
