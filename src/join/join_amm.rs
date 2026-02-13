use crate::types::amm_events::AmmEvent;
use crate::types::pubkey_borsh::Pubkey as BorshPubkey;
use crate::idl::match_ix::InstructionContext;

#[derive(Debug, Clone)]
pub struct AmmEventContext {
    pub signature: String,
    pub log_index: usize,
    pub raw_log: String,
    pub event: AmmEvent,
}

#[derive(Debug, Clone)]
pub struct JoinedAmmAction {
    pub ix: InstructionContext,
    pub event: AmmEventContext,
}

fn pk_to_string(pk: &BorshPubkey) -> String {
    pk.to_string()
}

pub fn join_amm_ix_and_events(
    ix_contexts: &[InstructionContext],
    events: &[AmmEventContext],
) -> Vec<JoinedAmmAction> {
    let mut joined = Vec::new();

    for ev_ctx in events {
        match &ev_ctx.event {
            AmmEvent::Sell(ev) => {
                let pool = pk_to_string(&ev.pool);
                let user = pk_to_string(&ev.user);

                if let Some(ix_ctx) = ix_contexts.iter().find(|ix| {
                    ix.ix_name == "sell"
                        && ix.accounts.get("pool").map(|s| s.as_str()) == Some(pool.as_str())
                        && ix.accounts.get("user").map(|s| s.as_str()) == Some(user.as_str())
                }) {
                    joined.push(JoinedAmmAction {
                        ix: ix_ctx.clone(),
                        event: ev_ctx.clone(),
                    });
                }
            }

            AmmEvent::Buy(ev) => {
                let pool = pk_to_string(&ev.pool);
                let user = pk_to_string(&ev.user);
                let ix_name = ev.ix_name.as_str();

                if let Some(ix_ctx) = ix_contexts.iter().find(|ix| {
                    ix.accounts.get("pool").map(|s| s.as_str()) == Some(pool.as_str())
                        && ix.accounts.get("user").map(|s| s.as_str()) == Some(user.as_str())
                        && ix.ix_name == ix_name
                }) {
                    joined.push(JoinedAmmAction {
                        ix: ix_ctx.clone(),
                        event: ev_ctx.clone(),
                    });
                }
            }
        }
    }

    joined
}
