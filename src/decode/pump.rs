use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use borsh::BorshDeserialize;
use std::io;

use crate::types::pump_events::{
    CollectCreatorFeeEvent, CompleteEvent, CompletePumpAmmMigrationEvent, CreateEvent, PumpEvent,
    SetMetaplexCreatorEvent, TradeEvent,
};

// Pumpfun Events discriminators
const TRADE_EVENT_DISCRIMINATOR: [u8; 8] = [189, 219, 127, 211, 78, 230, 97, 238];
const CREATE_EVENT_DISCRIMINATOR: [u8; 8] = [27, 114, 169, 77, 222, 235, 99, 118];
const SETMETAPLEX_CREATOR_EVENT_DISCRIMINATOR: [u8; 8] = [142, 203, 6, 32, 127, 105, 191, 162];
const COMPLETE_PUMPAMM_MIGRATION_EVENT_DISCRIMINATOR: [u8; 8] = [189, 233, 93, 185, 92, 148, 234, 148];
const COMPLETE_EVENT_DISCRIMINATOR: [u8; 8] = [95, 114, 97, 156, 212, 46, 152, 8];
const COLLECT_CREATORFEE_EVENT_DISCRIMINATOR: [u8; 8] = [122, 2, 127, 1, 14, 191, 12, 175];

fn decode_event_partial<T: BorshDeserialize>(
    payload: &[u8],
    base64_part: &str,
    disc_name: &str,
) -> Result<T> {
    let mut slice: &[u8] = payload;

    match T::deserialize(&mut slice) {
        Ok(ev) => Ok(ev),
        Err(e) => {
            let msg = format!(
                "borsh decode failed for {disc_name}: {e}; payload_len={}; base64_prefix={}",
                payload.len(),
                &base64_part[..base64_part.len().min(80)]
            );
            Err(anyhow!(io::Error::new(io::ErrorKind::InvalidData, msg)))
        }
    }
}

pub fn decode_pump_event_from_log(line: &str) -> Result<Option<PumpEvent>> {
    let prefix = "Program data: ";
    let base64_part = match line.strip_prefix(prefix) {
        Some(s) => s.trim(),
        None => return Ok(None),
    };

    if base64_part.starts_with("Synopsis ") {
        return Ok(None);
    }

    let bytes = match STANDARD.decode(base64_part) {
        Ok(b) => b,
        Err(_) => return Ok(None),
    };

    if bytes.len() < 8 {
        return Ok(None);
    }

    let (disc, payload) = bytes.split_at(8);

    let ev = if disc == TRADE_EVENT_DISCRIMINATOR {
        PumpEvent::Trade(decode_event_partial::<TradeEvent>(payload, base64_part, "TradeEvent")?)
    } else if disc == CREATE_EVENT_DISCRIMINATOR {
        PumpEvent::Create(decode_event_partial::<CreateEvent>(payload, base64_part, "CreateEvent")?)
    } else if disc == SETMETAPLEX_CREATOR_EVENT_DISCRIMINATOR {
        PumpEvent::SetMetaplexCreator(decode_event_partial::<SetMetaplexCreatorEvent>(
            payload,
            base64_part,
            "SetMetaplexCreatorEvent",
        )?)
    } else if disc == COMPLETE_PUMPAMM_MIGRATION_EVENT_DISCRIMINATOR {
        PumpEvent::CompletePumpAmmMigration(decode_event_partial::<CompletePumpAmmMigrationEvent>(
            payload,
            base64_part,
            "CompletePumpAmmMigrationEvent",
        )?)
    } else if disc == COMPLETE_EVENT_DISCRIMINATOR {
        PumpEvent::Complete(decode_event_partial::<CompleteEvent>(payload, base64_part, "CompleteEvent")?)
    } else if disc == COLLECT_CREATORFEE_EVENT_DISCRIMINATOR {
        PumpEvent::CollectCreatorFee(decode_event_partial::<CollectCreatorFeeEvent>(
            payload,
            base64_part,
            "CollectCreatorFeeEvent",
        )?)
    } else {
        return Ok(None);
    };

    Ok(Some(ev))
}
