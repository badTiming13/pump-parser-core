use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use borsh::BorshDeserialize;
use std::io::Cursor;
use tracing::{debug, warn};

use crate::types::amm_events::{AmmEvent, BuyEvent, SellEvent};

// Pumpswap Events discriminators
const BUY_EVENT_DISCRIMINATOR: [u8; 8] = [103, 244, 82, 31, 44, 245, 119, 119];
const SELL_EVENT_DISCRIMINATOR: [u8; 8] = [62, 47, 55, 10, 165, 3, 220, 42];

fn ascii_preview(bytes: &[u8], max: usize) -> String {
    let slice = if bytes.len() > max { &bytes[..max] } else { bytes };
    slice.iter()
        .map(|&c| if (32..=126).contains(&c) { c as char } else { '.' })
        .collect()
}

fn is_zero_padding_16(leftover: &[u8]) -> bool {
    leftover.len() == 16 && leftover.iter().all(|&b| b == 0)
}

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
    let disc_hex = hex::encode(disc);

    // Контекстная отладка (можешь убрать, если уже не нужно)
    if payload.len() >= 64 {
        let tail = &payload[payload.len().saturating_sub(96)..];
        debug!(
            disc_hex=%disc_hex,
            disc_bytes=?disc,
            raw_len=bytes.len(),
            payload_len=payload.len(),
            tail_ascii=%ascii_preview(tail, 96),
            "amm decode: seen program data payload"
        );
    }

    // BUY
    if disc == BUY_EVENT_DISCRIMINATOR {
        let mut cur = Cursor::new(payload);
        match BuyEvent::deserialize_reader(&mut cur) {
            Ok(event) => {
                let used = cur.position() as usize;
                let leftover = payload.len().saturating_sub(used);

                if leftover > 0 {
                    let left = &payload[used..];

                    if is_zero_padding_16(left) {
                        // Это нормальный padding (не шумим WARN-ами)
                        debug!(
                            disc="BUY",
                            disc_hex=%disc_hex,
                            payload_len=payload.len(),
                            used_len=used,
                            leftover_len=leftover,
                            "amm decode: BUY decoded (has 16B zero padding)"
                        );
                    } else {
                        warn!(
                            disc="BUY",
                            disc_hex=%disc_hex,
                            payload_len=payload.len(),
                            used_len=used,
                            leftover_len=leftover,
                            trailing_hex=%hex::encode(left),
                            trailing_ascii=%ascii_preview(left, 64),
                            "amm decode: BUY decoded but has unexpected trailing bytes"
                        );
                    }
                }

                return Ok(Some(AmmEvent::Buy(event)));
            }
            Err(e) => {
                warn!(
                    disc="BUY",
                    disc_hex=%disc_hex,
                    payload_len=payload.len(),
                    error=%e,
                    "amm decode: BUY discriminator matched but borsh deserialize failed"
                );
                return Ok(None);
            }
        }
    }

    // SELL
    if disc == SELL_EVENT_DISCRIMINATOR {
        let mut cur = Cursor::new(payload);
        match SellEvent::deserialize_reader(&mut cur) {
            Ok(event) => {
                let used = cur.position() as usize;
                let leftover = payload.len().saturating_sub(used);

                if leftover > 0 {
                    let left = &payload[used..];

                    if is_zero_padding_16(left) {
                        debug!(
                            disc="SELL",
                            disc_hex=%disc_hex,
                            payload_len=payload.len(),
                            used_len=used,
                            leftover_len=leftover,
                            "amm decode: SELL decoded (has 16B zero padding)"
                        );
                    } else {
                        warn!(
                            disc="SELL",
                            disc_hex=%disc_hex,
                            payload_len=payload.len(),
                            used_len=used,
                            leftover_len=leftover,
                            trailing_hex=%hex::encode(left),
                            trailing_ascii=%ascii_preview(left, 64),
                            "amm decode: SELL decoded but has unexpected trailing bytes"
                        );
                    }
                }

                return Ok(Some(AmmEvent::Sell(event)));
            }
            Err(e) => {
                warn!(
                    disc="SELL",
                    disc_hex=%disc_hex,
                    payload_len=payload.len(),
                    error=%e,
                    "amm decode: SELL discriminator matched but borsh deserialize failed"
                );
                return Ok(None);
            }
        }
    }

    Ok(None)
}
