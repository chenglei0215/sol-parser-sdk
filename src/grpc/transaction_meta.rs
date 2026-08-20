//! Yellowstone [`Transaction`] / [`TransactionStatusMeta`] 通用工具。
//!
//! 不依赖 DEX 日志或指令解析，适用于：mentions 订阅后的 SOL/SPL 转账分析、审计、风控等。

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::instr::read_pubkey_fast;
use crate::DexEvent;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use yellowstone_grpc_proto::prelude::{TokenBalance, Transaction, TransactionStatusMeta};

/// 32 字节公钥 → base58 地址字符串。
#[inline]
pub fn pubkey_bytes_to_bs58(bytes: &[u8]) -> Option<String> {
    let a: [u8; 32] = bytes.try_into().ok()?;
    Some(solana_sdk::pubkey::Pubkey::from(a).to_string())
}

/// 消息静态 `account_keys` + meta 中 `loaded_writable_addresses` / `loaded_readonly_addresses`，
/// 顺序与 `pre_balances` / `post_balances` 对齐。
pub fn collect_account_keys_bs58(
    tx: &Transaction,
    meta: &TransactionStatusMeta,
) -> Option<Vec<String>> {
    let msg = tx.message.as_ref()?;
    let mut keys: Vec<String> =
        msg.account_keys.iter().filter_map(|b| pubkey_bytes_to_bs58(b.as_slice())).collect();
    for b in &meta.loaded_writable_addresses {
        keys.push(pubkey_bytes_to_bs58(b)?);
    }
    for b in &meta.loaded_readonly_addresses {
        keys.push(pubkey_bytes_to_bs58(b)?);
    }
    Some(keys)
}

/// 每个账户索引的 lamports 变化（post - pre）。
#[inline]
pub fn lamport_balance_deltas(meta: &TransactionStatusMeta) -> Vec<i128> {
    meta.pre_balances
        .iter()
        .zip(meta.post_balances.iter())
        .map(|(pre, post)| *post as i128 - *pre as i128)
        .collect()
}

/// 启发式原生 SOL：对 `watched_bs58` 中出现的账户，若 lamports 净减少 ≥ `min_outflow_lamports`，
/// 再与其它索引配对，要求对方 delta ≥ `min_outflow_lamports/2`（与常见 mentions 转账监控一致）。
pub fn heuristic_sol_counterparties_for_watched_keys(
    account_keys_bs58: &[String],
    lamport_deltas: &[i128],
    watched_bs58: &HashSet<&str>,
    min_outflow_lamports: u64,
) -> Vec<(String, String)> {
    let min_l = min_outflow_lamports as i128;
    let mut pairs = Vec::new();
    for (i, key) in account_keys_bs58.iter().enumerate() {
        if !watched_bs58.contains(key.as_str()) {
            continue;
        }
        let d = lamport_deltas.get(i).copied().unwrap_or(0);
        if d >= -min_l {
            continue;
        }
        for (j, dj) in lamport_deltas.iter().enumerate() {
            if i == j || *dj <= min_l / 2 {
                continue;
            }
            pairs.push((key.clone(), account_keys_bs58[j].clone()));
        }
    }
    pairs
}

/// 汇总「监控地址」在一笔交易中的转出对手方（原生 SOL 启发式 + SPL token balance 启发式）。
///
/// 返回 `None` 当账户 key 与 balance 数组长度不一致。
pub fn collect_watch_transfer_counterparty_pairs(
    tx: &Transaction,
    meta: &TransactionStatusMeta,
    watched_bs58: &[String],
    min_native_outflow_lamports: u64,
    spl_min_watch_decrease_raw: u64,
) -> Option<Vec<(String, String)>> {
    let keys = collect_account_keys_bs58(tx, meta)?;
    let n = keys.len();
    if meta.pre_balances.len() != n || meta.post_balances.len() != n {
        return None;
    }
    let deltas = lamport_balance_deltas(meta);
    let watched_h: HashSet<&str> = watched_bs58.iter().map(|s| s.as_str()).collect();

    let mut pairs = heuristic_sol_counterparties_for_watched_keys(
        &keys,
        &deltas,
        &watched_h,
        min_native_outflow_lamports,
    );
    for w in watched_bs58 {
        pairs.extend(spl_token_counterparty_by_owner(meta, w, spl_min_watch_decrease_raw));
    }
    pairs.sort_by(|a, b| a.1.cmp(&b.1));
    pairs.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);
    Some(pairs)
}

/// `TokenBalance.ui_token_amount.amount` 解析为原始整数；失败为 0。
#[inline]
pub fn token_balance_raw_amount(t: &TokenBalance) -> u64 {
    t.ui_token_amount.as_ref().and_then(|u| u.amount.parse().ok()).unwrap_or(0)
}

/// SPL：对给定 owner（TokenBalance.owner，base58），当其某 mint 上余额净减少 ≥ `min_watch_decrease_raw` 时，
/// 找出同 mint 下余额增加的其它 owner，返回 `(watch_owner, counterparty_owner)`。
///
/// 用于启发式「谁转给谁」配对（非链上 Transfer 事件级精确解析）。
pub fn spl_token_counterparty_by_owner(
    meta: &TransactionStatusMeta,
    watch_owner_bs58: &str,
    min_watch_decrease_raw: u64,
) -> Vec<(String, String)> {
    use std::collections::{HashMap, HashSet};

    let pre = meta.pre_token_balances.as_slice();
    let post = meta.post_token_balances.as_slice();

    let mut pre_m: HashMap<(String, String), u64> = HashMap::new();
    for b in pre {
        if b.owner.is_empty() {
            continue;
        }
        let k = (b.mint.clone(), b.owner.clone());
        *pre_m.entry(k).or_insert(0) += token_balance_raw_amount(b);
    }
    let mut post_m: HashMap<(String, String), u64> = HashMap::new();
    for b in post {
        if b.owner.is_empty() {
            continue;
        }
        let k = (b.mint.clone(), b.owner.clone());
        *post_m.entry(k).or_insert(0) += token_balance_raw_amount(b);
    }

    let mut mints = HashSet::new();
    for (m, o) in pre_m.keys() {
        if o == watch_owner_bs58 {
            mints.insert(m.clone());
        }
    }
    for (m, o) in post_m.keys() {
        if o == watch_owner_bs58 {
            mints.insert(m.clone());
        }
    }

    let mut out = Vec::new();
    let min_l = min_watch_decrease_raw;
    for mint in mints {
        let w_pre = pre_m.get(&(mint.clone(), watch_owner_bs58.to_string())).copied().unwrap_or(0);
        let w_post =
            post_m.get(&(mint.clone(), watch_owner_bs58.to_string())).copied().unwrap_or(0);
        let lost = w_pre.saturating_sub(w_post);
        if lost < min_l.max(1) {
            continue;
        }
        for ((m, owner), po) in &post_m {
            if m != &mint || owner == watch_owner_bs58 {
                continue;
            }
            let pr = pre_m.get(&(mint.clone(), owner.clone())).copied().unwrap_or(0);
            if *po > pr {
                out.push((watch_owner_bs58.to_string(), owner.clone()));
            }
        }
    }
    out.sort_by(|a, b| a.1.cmp(&b.1));
    out.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);
    out
}

/// 仅消息头里的静态 `account_keys`（与 ShredStream `VersionedTransaction::static_account_keys()` 语义对齐；
/// 不含 ALT 加载地址）。
#[inline]
pub fn yellowstone_static_account_keys_arc(tx: &Option<Transaction>) -> Arc<[Pubkey]> {
    let Some(t) = tx.as_ref() else {
        return Arc::from(Vec::<Pubkey>::new().into_boxed_slice());
    };
    let Some(msg) = t.message.as_ref() else {
        return Arc::from(Vec::<Pubkey>::new().into_boxed_slice());
    };
    let keys: Vec<Pubkey> =
        msg.account_keys.iter().map(|bytes| read_pubkey_fast(bytes.as_slice())).collect();
    Arc::from(keys.into_boxed_slice())
}

/// Yellowstone 交易的 fee payer（消息静态账户第 0 位）。
#[inline]
pub fn yellowstone_fee_payer(tx: &Option<Transaction>) -> Pubkey {
    tx.as_ref()
        .and_then(|t| t.message.as_ref())
        .and_then(|msg| msg.account_keys.first())
        .map_or(Pubkey::default(), |bytes| read_pubkey_fast(bytes.as_slice()))
}

/// 将外层交易 fee payer 补充到 PumpFun trade 事件。
///
/// 该字段在 log/instruction 去重完成后统一填充，避免合并时被默认值覆盖。
#[inline]
pub(crate) fn fill_pumpfun_transaction_fee_payer(
    events: &mut [DexEvent],
    tx: &Option<Transaction>,
) {
    let fee_payer = yellowstone_fee_payer(tx);
    if fee_payer == Pubkey::default() {
        return;
    }
    for event in events {
        match event {
            DexEvent::PumpFunTrade(e)
            | DexEvent::PumpFunBuy(e)
            | DexEvent::PumpFunSell(e)
            | DexEvent::PumpFunBuyExactSolIn(e) => {
                e.transaction_fee_payer = fee_payer;
            }
            _ => {}
        }
    }
}

/// 用交易 token balance 中的 ATA owner 修正 instruction 路 PumpFun 事件用户。
///
/// 批量/路由交易的 fee payer 和每段 PumpFun CPI 的真实用户可能不同。instruction
/// 账户解析偶尔会把外层账户填进 `user`，但 `associated_user` 对应 token balance 的
/// owner 是该段交易的真实钱包。该修正在 log/ix 去重前执行，使 instruction 事件能
/// 与权威的 TradeEvent log 正确配对，同时保留同签名中的多个真实用户。
pub(crate) fn fill_pumpfun_instruction_users_from_token_balances(
    events: &mut [DexEvent],
    tx: &Option<Transaction>,
    meta: &TransactionStatusMeta,
) {
    if !events.iter().any(|event| {
        matches!(
            event,
            DexEvent::PumpFunTrade(_)
                | DexEvent::PumpFunBuy(_)
                | DexEvent::PumpFunSell(_)
                | DexEvent::PumpFunBuyExactSolIn(_)
        )
    }) {
        return;
    }
    let Some(keys) = collect_account_keys_pubkeys(tx, meta) else {
        return;
    };

    let mut owners: HashMap<(Pubkey, Pubkey), Pubkey> = HashMap::new();
    for balance in meta.pre_token_balances.iter().chain(meta.post_token_balances.iter()) {
        let Some(account) = keys.get(balance.account_index as usize).copied() else {
            continue;
        };
        let Ok(mint) = balance.mint.parse::<Pubkey>() else {
            continue;
        };
        let Ok(owner) = balance.owner.parse::<Pubkey>() else {
            continue;
        };
        if owner != Pubkey::default() {
            owners.insert((account, mint), owner);
        }
    }

    for event in events {
        let trade = match event {
            DexEvent::PumpFunTrade(e)
            | DexEvent::PumpFunBuy(e)
            | DexEvent::PumpFunSell(e)
            | DexEvent::PumpFunBuyExactSolIn(e) => e,
            _ => continue,
        };
        if trade.associated_user == Pubkey::default() {
            continue;
        }
        if let Some(owner) = owners.get(&(trade.associated_user, trade.mint)) {
            if trade.user != *owner {
                tracing::info!(
                    target: "parser_user_fix",
                    "pumpfun_user_attribution_corrected: signature={} mint={} old_user={} real_user={} associated_user={} slot={} tx_index={}",
                    trade.metadata.signature,
                    trade.mint,
                    trade.user,
                    owner,
                    trade.associated_user,
                    trade.metadata.slot,
                    trade.metadata.tx_index
                );
            }
            trade.user = *owner;
        }
    }
}

/// 用外层签名者在目标 mint 上的净减仓，修正聚合路由 PumpFun 卖出的真实用户。
///
/// 聚合器通常先把 leader 的 token 转入临时账户，再以内层 PDA 调用 PumpFun Sell。
/// 此时 TradeEvent.user / associated_user owner 都是路由账户，无法与真实 leader 对上；
/// 但交易 meta 里仍能看到某个外层签名者的该 mint 总余额恰好减少。只有当同一 mint
/// 存在唯一、且净减仓足以覆盖本交易 PumpFun 卖出量的签名者时才修正，歧义场景保持原值。
pub(crate) fn fill_pumpfun_routed_sell_users_from_signer_token_deltas(
    events: &mut [DexEvent],
    tx: &Option<Transaction>,
    meta: &TransactionStatusMeta,
) {
    let Some(message) = tx.as_ref().and_then(|transaction| transaction.message.as_ref()) else {
        return;
    };
    let required_signatures = message
        .header
        .as_ref()
        .map(|header| header.num_required_signatures as usize)
        .unwrap_or(0)
        .min(message.account_keys.len());
    if required_signatures == 0 {
        return;
    }

    let signers: HashSet<Pubkey> = message.account_keys[..required_signatures]
        .iter()
        .map(|bytes| read_pubkey_fast(bytes.as_slice()))
        .filter(|key| *key != Pubkey::default())
        .collect();
    if signers.is_empty() {
        return;
    }

    let mut total_sell_by_mint = HashMap::<Pubkey, u128>::new();
    for event in events.iter() {
        let trade = match event {
            DexEvent::PumpFunTrade(e)
            | DexEvent::PumpFunBuy(e)
            | DexEvent::PumpFunSell(e)
            | DexEvent::PumpFunBuyExactSolIn(e) => e,
            _ => continue,
        };
        if !trade.is_buy && trade.token_amount > 0 {
            *total_sell_by_mint.entry(trade.mint).or_insert(0) += trade.token_amount as u128;
        }
    }
    if total_sell_by_mint.is_empty() {
        return;
    }

    let mut pre_by_owner_mint = HashMap::<(Pubkey, Pubkey), u128>::new();
    let mut post_by_owner_mint = HashMap::<(Pubkey, Pubkey), u128>::new();
    for (balances, target) in [
        (meta.pre_token_balances.as_slice(), &mut pre_by_owner_mint),
        (meta.post_token_balances.as_slice(), &mut post_by_owner_mint),
    ] {
        for balance in balances {
            let Ok(owner) = balance.owner.parse::<Pubkey>() else {
                continue;
            };
            if !signers.contains(&owner) {
                continue;
            }
            let Ok(mint) = balance.mint.parse::<Pubkey>() else {
                continue;
            };
            *target.entry((owner, mint)).or_insert(0) += token_balance_raw_amount(balance) as u128;
        }
    }

    let mut attributed_signer_by_mint = HashMap::<Pubkey, Pubkey>::new();
    for (mint, total_sell) in total_sell_by_mint {
        let mut candidates = signers.iter().filter_map(|signer| {
            let key = (*signer, mint);
            let pre = pre_by_owner_mint.get(&key).copied().unwrap_or(0);
            let post = post_by_owner_mint.get(&key).copied().unwrap_or(0);
            let decrease = pre.saturating_sub(post);
            (decrease >= total_sell).then_some(*signer)
        });
        let Some(candidate) = candidates.next() else {
            continue;
        };
        if candidates.next().is_none() {
            attributed_signer_by_mint.insert(mint, candidate);
        }
    }

    for event in events {
        let trade = match event {
            DexEvent::PumpFunTrade(e)
            | DexEvent::PumpFunBuy(e)
            | DexEvent::PumpFunSell(e)
            | DexEvent::PumpFunBuyExactSolIn(e) => e,
            _ => continue,
        };
        if trade.is_buy {
            continue;
        }
        let Some(real_user) = attributed_signer_by_mint.get(&trade.mint).copied() else {
            continue;
        };
        if trade.user != real_user {
            tracing::info!(
                target: "parser_user_fix",
                "pumpfun_routed_sell_user_corrected: signature={} mint={} old_user={} real_user={} token_amount={} slot={} tx_index={}",
                trade.metadata.signature,
                trade.mint,
                trade.user,
                real_user,
                trade.token_amount,
                trade.metadata.slot,
                trade.metadata.tx_index,
            );
            trade.user = real_user;
        }
    }
}

fn collect_account_keys_pubkeys(
    tx: &Option<Transaction>,
    meta: &TransactionStatusMeta,
) -> Option<Vec<Pubkey>> {
    let msg = tx.as_ref()?.message.as_ref()?;
    let mut keys = Vec::with_capacity(
        msg.account_keys.len()
            + meta.loaded_writable_addresses.len()
            + meta.loaded_readonly_addresses.len(),
    );
    for bytes in msg
        .account_keys
        .iter()
        .chain(meta.loaded_writable_addresses.iter())
        .chain(meta.loaded_readonly_addresses.iter())
    {
        let raw: [u8; 32] = bytes.as_slice().try_into().ok()?;
        keys.push(Pubkey::from(raw));
    }
    Some(keys)
}

/// Yellowstone 交易签名原始字节（64）→ `solana_sdk::signature::Signature`。
#[inline]
pub fn try_yellowstone_signature(sig: &[u8]) -> Option<Signature> {
    if sig.len() != 64 {
        return None;
    }
    let a: [u8; 64] = sig.try_into().ok()?;
    Some(Signature::from(a))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::events::PumpFunTradeEvent;
    use yellowstone_grpc_proto::prelude::{Message, MessageHeader, TokenBalance, UiTokenAmount};

    fn token_balance(account_index: u32, mint: Pubkey, owner: Pubkey, amount: u64) -> TokenBalance {
        TokenBalance {
            account_index,
            mint: mint.to_string(),
            owner: owner.to_string(),
            ui_token_amount: Some(UiTokenAmount {
                amount: amount.to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn fills_outer_fee_payer_without_overwriting_pumpfun_user() {
        let fee_payer = Pubkey::new_unique();
        let inner_user = Pubkey::new_unique();
        let tx = Some(Transaction {
            message: Some(Message {
                account_keys: vec![fee_payer.to_bytes().to_vec()],
                ..Default::default()
            }),
            ..Default::default()
        });
        let mut events = vec![DexEvent::PumpFunTrade(PumpFunTradeEvent {
            user: inner_user,
            ..Default::default()
        })];

        fill_pumpfun_transaction_fee_payer(&mut events, &tx);

        let DexEvent::PumpFunTrade(event) = &events[0] else {
            panic!("expected PumpFunTrade");
        };
        assert_eq!(event.user, inner_user);
        assert_eq!(event.transaction_fee_payer, fee_payer);
    }

    #[test]
    fn restores_distinct_users_for_batched_pumpfun_trades() {
        let fee_payer = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let ata_a = Pubkey::new_unique();
        let ata_b = Pubkey::new_unique();
        let owner_a = Pubkey::new_unique();
        let owner_b = Pubkey::new_unique();
        let tx = Some(Transaction {
            message: Some(Message {
                account_keys: vec![
                    fee_payer.to_bytes().to_vec(),
                    ata_a.to_bytes().to_vec(),
                    ata_b.to_bytes().to_vec(),
                ],
                ..Default::default()
            }),
            ..Default::default()
        });
        let token_balance = |account_index, owner: Pubkey| TokenBalance {
            account_index,
            mint: mint.to_string(),
            owner: owner.to_string(),
            ..Default::default()
        };
        let meta = TransactionStatusMeta {
            pre_token_balances: vec![token_balance(1, owner_a), token_balance(2, owner_b)],
            ..Default::default()
        };
        let wrong_outer_user = fee_payer;
        let mut events = vec![
            DexEvent::PumpFunSell(PumpFunTradeEvent {
                mint,
                user: wrong_outer_user,
                associated_user: ata_a,
                is_buy: false,
                ..Default::default()
            }),
            DexEvent::PumpFunSell(PumpFunTradeEvent {
                mint,
                user: wrong_outer_user,
                associated_user: ata_b,
                is_buy: false,
                ..Default::default()
            }),
        ];

        fill_pumpfun_instruction_users_from_token_balances(&mut events, &tx, &meta);

        let users: Vec<Pubkey> = events
            .iter()
            .map(|event| match event {
                DexEvent::PumpFunSell(e) => e.user,
                _ => panic!("expected PumpFunSell"),
            })
            .collect();
        assert_eq!(users, vec![owner_a, owner_b]);

        let log_events = vec![
            DexEvent::PumpFunTrade(PumpFunTradeEvent {
                mint,
                user: owner_a,
                is_buy: false,
                ..Default::default()
            }),
            DexEvent::PumpFunTrade(PumpFunTradeEvent {
                mint,
                user: owner_b,
                is_buy: false,
                ..Default::default()
            }),
        ];
        let merged =
            crate::grpc::log_instr_dedup::dedupe_log_instruction_events(log_events, events);
        assert_eq!(merged.len(), 2, "批量交易的 log/ix 双路事件应各合并为一条");
        let merged_users: Vec<Pubkey> = merged
            .iter()
            .map(|event| match event {
                DexEvent::PumpFunTrade(e) => e.user,
                _ => panic!("expected canonical PumpFunTrade"),
            })
            .collect();
        assert_eq!(merged_users, vec![owner_a, owner_b]);
    }

    #[test]
    fn restores_routed_sell_user_from_unique_signer_token_decrease() {
        let fee_payer = Pubkey::new_unique();
        let leader = Pubkey::new_unique();
        let router = Pubkey::new_unique();
        let leader_ata = Pubkey::new_unique();
        let router_ata = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let tx = Some(Transaction {
            message: Some(Message {
                header: Some(MessageHeader { num_required_signatures: 2, ..Default::default() }),
                account_keys: vec![
                    fee_payer.to_bytes().to_vec(),
                    leader.to_bytes().to_vec(),
                    leader_ata.to_bytes().to_vec(),
                    router_ata.to_bytes().to_vec(),
                ],
                ..Default::default()
            }),
            ..Default::default()
        });
        let meta = TransactionStatusMeta {
            pre_token_balances: vec![
                token_balance(2, mint, leader, 1_000),
                token_balance(3, mint, router, 0),
            ],
            post_token_balances: vec![
                token_balance(2, mint, leader, 500),
                token_balance(3, mint, router, 0),
            ],
            ..Default::default()
        };
        let mut events = vec![DexEvent::PumpFunSell(PumpFunTradeEvent {
            mint,
            user: router,
            associated_user: router_ata,
            token_amount: 500,
            is_buy: false,
            ..Default::default()
        })];

        fill_pumpfun_routed_sell_users_from_signer_token_deltas(&mut events, &tx, &meta);

        let DexEvent::PumpFunSell(event) = &events[0] else {
            panic!("expected PumpFunSell");
        };
        assert_eq!(event.user, leader);
    }

    #[test]
    fn routed_sell_attribution_keeps_user_when_decreasing_owner_is_not_signer() {
        let fee_payer = Pubkey::new_unique();
        let leader = Pubkey::new_unique();
        let router = Pubkey::new_unique();
        let router_ata = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let tx = Some(Transaction {
            message: Some(Message {
                header: Some(MessageHeader { num_required_signatures: 1, ..Default::default() }),
                account_keys: vec![fee_payer.to_bytes().to_vec(), router_ata.to_bytes().to_vec()],
                ..Default::default()
            }),
            ..Default::default()
        });
        let meta = TransactionStatusMeta {
            pre_token_balances: vec![token_balance(1, mint, leader, 1_000)],
            post_token_balances: vec![token_balance(1, mint, leader, 500)],
            ..Default::default()
        };
        let mut events = vec![DexEvent::PumpFunSell(PumpFunTradeEvent {
            mint,
            user: router,
            associated_user: router_ata,
            token_amount: 500,
            is_buy: false,
            ..Default::default()
        })];

        fill_pumpfun_routed_sell_users_from_signer_token_deltas(&mut events, &tx, &meta);

        let DexEvent::PumpFunSell(event) = &events[0] else {
            panic!("expected PumpFunSell");
        };
        assert_eq!(event.user, router);
    }

    #[test]
    fn routed_sell_attribution_keeps_user_when_multiple_signers_are_ambiguous() {
        let signer_a = Pubkey::new_unique();
        let signer_b = Pubkey::new_unique();
        let router = Pubkey::new_unique();
        let ata_a = Pubkey::new_unique();
        let ata_b = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let tx = Some(Transaction {
            message: Some(Message {
                header: Some(MessageHeader { num_required_signatures: 2, ..Default::default() }),
                account_keys: vec![
                    signer_a.to_bytes().to_vec(),
                    signer_b.to_bytes().to_vec(),
                    ata_a.to_bytes().to_vec(),
                    ata_b.to_bytes().to_vec(),
                ],
                ..Default::default()
            }),
            ..Default::default()
        });
        let meta = TransactionStatusMeta {
            pre_token_balances: vec![
                token_balance(2, mint, signer_a, 1_000),
                token_balance(3, mint, signer_b, 1_000),
            ],
            post_token_balances: vec![
                token_balance(2, mint, signer_a, 500),
                token_balance(3, mint, signer_b, 500),
            ],
            ..Default::default()
        };
        let mut events = vec![DexEvent::PumpFunSell(PumpFunTradeEvent {
            mint,
            user: router,
            token_amount: 500,
            is_buy: false,
            ..Default::default()
        })];

        fill_pumpfun_routed_sell_users_from_signer_token_deltas(&mut events, &tx, &meta);

        let DexEvent::PumpFunSell(event) = &events[0] else {
            panic!("expected PumpFunSell");
        };
        assert_eq!(event.user, router);
    }
}
