use crate::idl::match_ix::InstructionContext;
use crate::types::pump_events::PumpEvent;
use crate::types::pubkey_borsh::Pubkey as BorshPubkey;

#[derive(Debug, Clone)]
pub struct EventContext {
    pub signature: String,
    pub log_index: usize,
    pub raw_log: String,
    pub event: PumpEvent,
}

#[derive(Debug, Clone)]
pub struct JoinedPumpAction {
    pub ix: InstructionContext,
    pub event: EventContext,
}

fn pk_to_string(pk: &BorshPubkey) -> String {
    pk.to_string()
}

pub fn join_pump_ix_and_events(
    ix_contexts: &[InstructionContext],
    events: &[EventContext],
) -> Vec<JoinedPumpAction> {
    let mut joined = Vec::new();

    for ev_ctx in events {
        match &ev_ctx.event {
            PumpEvent::Trade(ev) => {
                let mint = pk_to_string(&ev.mint);
                let user = pk_to_string(&ev.user);
                let ix_name = ev.ix_name.as_str(); // "buy" / "sell"

                if let Some(ix_ctx) = ix_contexts.iter().find(|ix| {
                    ix.ix_name == ix_name
                        && ix.accounts.get("mint").map(|s| s.as_str()) == Some(mint.as_str())
                        && ix.accounts.get("user").map(|s| s.as_str()) == Some(user.as_str())
                }) {
                    joined.push(JoinedPumpAction {
                        ix: ix_ctx.clone(),
                        event: ev_ctx.clone(),
                    });
                }
            }

            PumpEvent::Create(ev) => {
                let mint = pk_to_string(&ev.mint);
                let user = pk_to_string(&ev.user);
                let bonding_curve = pk_to_string(&ev.bonding_curve);

                if let Some(ix_ctx) = ix_contexts.iter().find(|ix| {
                    ix.ix_name.starts_with("create")
                        && ix.accounts.get("mint").map(|s| s.as_str()) == Some(mint.as_str())
                        && ix.accounts.get("user").map(|s| s.as_str()) == Some(user.as_str())
                        && ix.accounts.get("bonding_curve").map(|s| s.as_str())
                            == Some(bonding_curve.as_str())
                }) {
                    joined.push(JoinedPumpAction {
                        ix: ix_ctx.clone(),
                        event: ev_ctx.clone(),
                    });
                }
            }

            PumpEvent::CollectCreatorFee(ev) => {
                let creator = pk_to_string(&ev.creator);

                if let Some(ix_ctx) = ix_contexts.iter().find(|ix| {
                    ix.ix_name.starts_with("collect")
                        && ix.accounts.get("creator").map(|s| s.as_str()) == Some(creator.as_str())
                }) {
                    joined.push(JoinedPumpAction {
                        ix: ix_ctx.clone(),
                        event: ev_ctx.clone(),
                    });
                }
            }

            _ => {}
        }
    }

    joined
}
