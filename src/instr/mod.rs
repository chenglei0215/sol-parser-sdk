//! 指令解析器模块
//!
//! 包含所有 DEX 协议的指令解析器实现

pub mod meteora_amm;
pub mod meteora_damm;
pub mod meteora_dlmm;
pub mod orca_whirlpool;
pub mod program_ids;
pub mod pump;
pub mod pump_amm;
pub mod pump_fees;
pub mod raydium_amm;
pub mod raydium_clmm;
pub mod raydium_cpmm;
pub mod raydium_launchlab;
pub mod utils;

// Inner instruction 解析器（16字节 discriminator）
pub mod all_inner;
pub mod inner_common; // 通用零拷贝读取函数
pub mod pump_amm_inner; // PumpSwap inner instruction
pub mod pump_inner; // PumpFun inner instruction
pub mod raydium_clmm_inner; // Raydium CLMM inner instruction // 其他所有协议的 inner instruction（统一文件）
use crate::grpc::types::EventTypeFilter;
// 重新导出主要解析函数
pub use meteora_damm::parse_instruction as parse_meteora_damm_instruction;
pub use pump::parse_instruction as parse_pumpfun_instruction;
pub use pump_amm::parse_instruction as parse_pumpswap_instruction;
pub use raydium_launchlab::parse_instruction as parse_raydium_launchlab_instruction;

// 重新导出工具函数
pub use utils::*;

use crate::core::events::DexEvent;
use program_ids::*;
use solana_sdk::{pubkey::Pubkey, signature::Signature};

#[inline(always)]
fn filter_parsed_event(
    event: Option<DexEvent>,
    event_type_filter: Option<&EventTypeFilter>,
) -> Option<DexEvent> {
    let event = event?;
    if event_type_filter.map(|f| f.should_include_dex_event(&event)).unwrap_or(true) {
        Some(event)
    } else {
        None
    }
}

/// 统一的指令解析入口函数
#[inline]
pub fn parse_instruction_unified(
    instruction_data: &[u8],
    accounts: &[Pubkey],
    signature: Signature,
    slot: u64,
    tx_index: u64,
    block_time_us: Option<i64>,
    grpc_recv_us: i64,
    event_type_filter: Option<&EventTypeFilter>,
    program_id: &Pubkey,
) -> Option<DexEvent> {
    // 快速检查指令数据长度，避免无效解析
    if instruction_data.is_empty() {
        return None;
    }

    // 根据程序 ID 路由到相应的解析器，按使用频率排序

    // Pumpfun
    if *program_id == PUMPFUN_PROGRAM_ID {
        if event_type_filter.is_some() && !event_type_filter.unwrap().includes_pumpfun() {
            return None;
        }
        return filter_parsed_event(
            parse_pumpfun_instruction(
                instruction_data,
                accounts,
                signature,
                slot,
                tx_index,
                block_time_us,
                grpc_recv_us,
            ),
            event_type_filter,
        );
    }
    // PumpSwap (Pump AMM)
    else if *program_id == PUMPSWAP_PROGRAM_ID {
        if event_type_filter.is_some() && !event_type_filter.unwrap().includes_pumpswap() {
            return None;
        }
        return filter_parsed_event(
            parse_pumpswap_instruction(
                instruction_data,
                accounts,
                signature,
                slot,
                tx_index,
                block_time_us,
            ),
            event_type_filter,
        );
    }
    // Meteora DAMM
    else if *program_id == METEORA_DAMM_V2_PROGRAM_ID {
        if event_type_filter.is_some() && !event_type_filter.unwrap().includes_meteora_damm_v2() {
            return None;
        }
        return filter_parsed_event(
            parse_meteora_damm_instruction(
                instruction_data,
                accounts,
                signature,
                slot,
                tx_index,
                block_time_us,
                grpc_recv_us,
            ),
            event_type_filter,
        );
    }
    // Pump fees (`pfeeUx...`)
    else if *program_id == PUMP_FEES_PROGRAM_ID {
        if event_type_filter.is_some() && !event_type_filter.unwrap().includes_pump_fees() {
            return None;
        }
        return filter_parsed_event(
            crate::instr::pump_fees::parse_instruction(
                instruction_data,
                accounts,
                signature,
                slot,
                tx_index,
                block_time_us,
                grpc_recv_us,
            ),
            event_type_filter,
        );
    }
    // RaydiumLaunchlab / Raydium LaunchLab
    else if *program_id == RAYDIUM_LAUNCHLAB_PROGRAM_ID {
        if event_type_filter.is_some() && !event_type_filter.unwrap().includes_raydium_launchlab() {
            return None;
        }
        return filter_parsed_event(
            parse_raydium_launchlab_instruction(
                instruction_data,
                accounts,
                signature,
                slot,
                tx_index,
                block_time_us,
            ),
            event_type_filter,
        );
    }
    // Raydium CPMM
    else if *program_id == RAYDIUM_CPMM_PROGRAM_ID {
        if event_type_filter.is_some() && !event_type_filter.unwrap().includes_raydium_cpmm() {
            return None;
        }
        return filter_parsed_event(
            crate::instr::raydium_cpmm::parse_instruction(
                instruction_data,
                accounts,
                signature,
                slot,
                tx_index,
                block_time_us,
            ),
            event_type_filter,
        );
    }
    // Raydium CLMM
    else if *program_id == RAYDIUM_CLMM_PROGRAM_ID {
        if event_type_filter.is_some() && !event_type_filter.unwrap().includes_raydium_clmm() {
            return None;
        }
        return filter_parsed_event(
            crate::instr::raydium_clmm::parse_instruction(
                instruction_data,
                accounts,
                signature,
                slot,
                tx_index,
                block_time_us,
            ),
            event_type_filter,
        );
    }
    // Raydium AMM V4
    else if *program_id == RAYDIUM_AMM_V4_PROGRAM_ID {
        if event_type_filter.is_some() && !event_type_filter.unwrap().includes_raydium_amm_v4() {
            return None;
        }
        return filter_parsed_event(
            crate::instr::raydium_amm::parse_instruction(
                instruction_data,
                accounts,
                signature,
                slot,
                tx_index,
                block_time_us,
            ),
            event_type_filter,
        );
    }
    // Orca Whirlpool
    else if *program_id == ORCA_WHIRLPOOL_PROGRAM_ID {
        if event_type_filter.is_some() && !event_type_filter.unwrap().includes_orca_whirlpool() {
            return None;
        }
        return filter_parsed_event(
            crate::instr::orca_whirlpool::parse_instruction(
                instruction_data,
                accounts,
                signature,
                slot,
                tx_index,
                block_time_us,
            ),
            event_type_filter,
        );
    }
    // Meteora Pools / AMM
    else if *program_id == METEORA_POOLS_PROGRAM_ID {
        if event_type_filter.is_some() && !event_type_filter.unwrap().includes_meteora_pools() {
            return None;
        }
        return filter_parsed_event(
            crate::instr::meteora_amm::parse_instruction(
                instruction_data,
                accounts,
                signature,
                slot,
                tx_index,
                block_time_us,
            ),
            event_type_filter,
        );
    }
    // Meteora DLMM
    else if *program_id == METEORA_DLMM_PROGRAM_ID {
        if event_type_filter.is_some() && !event_type_filter.unwrap().includes_meteora_dlmm() {
            return None;
        }
        return filter_parsed_event(
            crate::instr::meteora_dlmm::parse_instruction(
                instruction_data,
                accounts,
                signature,
                slot,
                tx_index,
                block_time_us,
            ),
            event_type_filter,
        );
    }

    None
}
