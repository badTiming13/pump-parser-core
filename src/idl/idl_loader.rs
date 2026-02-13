use anyhow::Result;

use crate::types::pump_idl::PumpIdl;

pub fn load_idl_from_str(json: &str) -> Result<PumpIdl> {
    Ok(serde_json::from_str::<PumpIdl>(json)?)
}
