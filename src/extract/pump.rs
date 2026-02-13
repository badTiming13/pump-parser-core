use crate::decode::pump::decode_pump_event_from_log;
use crate::idl::collect::collect_instructions;
use crate::idl::match_ix::{collect_contexts, InstructionContext};
use crate::join::join_pump::{join_pump_ix_and_events, EventContext, JoinedPumpAction};
use crate::types::block_notification::BlockNotification;
use crate::types::pump_idl::PumpIdl;

/// Вынесли “технический пайплайн” Pump:
/// - collect ix (outer+inner) по program
/// - собрать ix contexts по IDL
/// - decode events из logs
/// - join ix+event -> JoinedPumpAction
///
/// Никаких Redis/DB тут нет.
#[derive(Debug)]
pub struct PumpExtracted {
    pub slot: u64,
    pub joined_actions: Vec<JoinedPumpAction>,
    pub migrate_ix: Vec<InstructionContext>, // чтобы приложение решало “что с этим делать”
}

pub fn extract_pump_block(
    notification: &BlockNotification,
    idl: &PumpIdl,
    pump_program_address: &str,
) -> Result<PumpExtracted, Box<dyn std::error::Error>> {
    let txs = &notification.params.result.value.block.transactions;
    let slot: u64 = notification.params.result.value.slot;

    if txs.is_empty() {
        return Ok(PumpExtracted {
            slot,
            joined_actions: vec![],
            migrate_ix: vec![],
        });
    }

    let mut out_joined: Vec<JoinedPumpAction> = Vec::new();
    let mut migrate_ix: Vec<InstructionContext> = Vec::new();

    for (tx_idx, tx) in txs.iter().enumerate() {
        let Some((outer, inner)) = collect_instructions(tx, pump_program_address) else {
            continue;
        };

        // пропускаем failed tx (как у тебя было)
        if tx.meta.err.is_some() {
            continue;
        }

        // contexts
        let ix_contexts = match collect_contexts(tx, idl, &outer, &inner, tx_idx) {
            Ok(v) => v,
            Err(_) => continue, // мягко пропускаем
        };

        // отдельно вытащим migrate ix (чтобы app отправлял сигнал)
        migrate_ix.extend(ix_contexts.iter().filter(|ix| ix.ix_name == "migrate").cloned());
        let signature = tx.transaction.signatures.get(0).cloned().unwrap_or_default();
        // events
        let mut event_contexts: Vec<EventContext> = Vec::new();
        for (log_idx, line) in tx.meta.log_messages.as_deref().unwrap_or(&[]).iter().enumerate() {
            let decoded = match decode_pump_event_from_log(line) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if let Some(event) = decoded {
                event_contexts.push(EventContext {
                    signature: signature.clone(),
                    log_index: log_idx,
                    raw_log: line.clone(),
                    event,
                });
            }
        }

        // join
        let joined = join_pump_ix_and_events(&ix_contexts, &event_contexts);
        out_joined.extend(joined);
    }

    Ok(PumpExtracted {
        slot,
        joined_actions: out_joined,
        migrate_ix,
    })
}
