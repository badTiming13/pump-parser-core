use crate::decode::amm::decode_amm_event_from_log;
use crate::idl::collect::collect_instructions;
use crate::idl::match_ix::collect_contexts;
use crate::join::join_amm::{join_amm_ix_and_events, AmmEventContext, JoinedAmmAction};
use crate::types::block_notification::BlockNotification;
use crate::types::pump_idl::PumpIdl;

/// Аналогично для AMM.
#[derive(Debug)]
pub struct AmmExtracted {
    pub slot: u64,
    pub joined_actions: Vec<JoinedAmmAction>,
}

pub fn extract_amm_block(
    notification: &BlockNotification,
    idl: &PumpIdl,
    amm_program_address: &str,
) -> Result<AmmExtracted, Box<dyn std::error::Error>> {
    let txs = &notification.params.result.value.block.transactions;
    let slot: u64 = notification.params.result.value.slot;

    if txs.is_empty() {
        return Ok(AmmExtracted {
            slot,
            joined_actions: vec![],
        });
    }

    let mut out_joined: Vec<JoinedAmmAction> = Vec::new();

    for (tx_idx, tx) in txs.iter().enumerate() {
        let Some((outer, inner)) = collect_instructions(tx, amm_program_address) else {
            continue;
        };

        if tx.meta.err.is_some() {
            continue;
        }

        let ix_contexts = match collect_contexts(tx, idl, &outer, &inner, tx_idx) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let signature = tx.transaction.signatures.get(0).cloned().unwrap_or_default();
        let mut ev_ctxs: Vec<AmmEventContext> = Vec::new();
        for (log_idx, line) in tx.meta.log_messages.as_deref().unwrap_or(&[]).iter().enumerate() {
            let decoded = match decode_amm_event_from_log(line) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if let Some(event) = decoded {
                ev_ctxs.push(AmmEventContext {
                    signature: signature.clone(),
                    log_index: log_idx,
                    raw_log: line.clone(),
                    event,
                });
            }
        }

        let joined = join_amm_ix_and_events(&ix_contexts, &ev_ctxs);
        out_joined.extend(joined);
    }

    Ok(AmmExtracted {
        slot,
        joined_actions: out_joined,
    })
}
