use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use borsh::BorshDeserialize;

use crate::types::amm_events::{AmmEvent, BuyEvent, SellEvent};

// Pumpswap Events discriminators
const BUY_EVENT_DISCRIMINATOR: [u8; 8] = [103, 244, 82, 31, 44, 245, 119, 119];
const SELL_EVENT_DISCRIMINATOR: [u8; 8] = [62, 47, 55, 10, 165, 3, 220, 42];

pub fn decode_amm_event_from_log(line: &str) -> Result<Option<AmmEvent>> {
    let prefix = "Program data: ";
    let data = match line.strip_prefix(prefix) {
        Some(s) => s.trim(),
        None => return Ok(None),
    };

    if data.starts_with("Synopsis ") {
        return Ok(None);
    }

    let bytes = match STANDARD.decode(data) {
        Ok(b) => b,
        Err(_) => return Ok(None),
    };

    if bytes.len() < 8 {
        return Ok(None);
    }

    let (disc, payload) = bytes.split_at(8);

    if disc == BUY_EVENT_DISCRIMINATOR {
        let event = BuyEvent::try_from_slice(payload)?;
        Ok(Some(AmmEvent::Buy(event)))
    } else if disc == SELL_EVENT_DISCRIMINATOR {
        let event = SellEvent::try_from_slice(payload)?;
        Ok(Some(AmmEvent::Sell(event)))
    } else {
        Ok(None)
    }
}
