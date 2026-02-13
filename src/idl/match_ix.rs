use bs58;
use std::collections::BTreeMap;

use crate::types::block_notification::{Instruction, Transaction};
use crate::types::pump_idl::{Instruction as IdlInstruction, PumpIdl};

pub type AccountMap = BTreeMap<String, String>;

#[derive(Debug, Clone)]
pub struct InstructionContext {
    pub ix_name: String,
    pub label: String,
    pub ix_index: usize,
    pub is_inner: bool,
    pub accounts: AccountMap,
}

/// Map instruction accounts:
/// IDL account name -> actual pubkey from transaction
pub fn map_ix_accounts(
    tx: &Transaction,
    ix: &Instruction,
    idl_ix: &IdlInstruction,
) -> Vec<(String, String)> {
    let all_accounts = crate::idl::collect::get_accounts(tx);
    let mut mapped = Vec::new();

    for (i, idl_acc) in idl_ix.accounts.iter().enumerate() {
        let Some(&account_idx) = ix.accounts.get(i) else {
            continue;
        };

        let idx = account_idx as usize;
        if let Some(pk) = all_accounts.get(idx) {
            let name = idl_acc
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("<unknown>")
                .to_string();
            mapped.push((name, (*pk).clone()));
        }
    }

    mapped
}

/// Decode ix.data, take discriminator, find matching instruction in IDL.
pub fn match_instruction<'a>(
    ix: &Instruction,
    idl: &'a PumpIdl,
) -> Result<Option<(&'a IdlInstruction, Vec<u8>)>, bs58::decode::Error> {
    let decoded = bs58::decode(&ix.data).into_vec()?;

    if decoded.len() < 8 {
        return Ok(None);
    }

    let discriminator = &decoded[..8];

    let maybe_ix = idl
        .instructions
        .iter()
        .find(|idl_ix| idl_ix.discriminator.as_slice() == discriminator);

    Ok(maybe_ix.map(|idl_ix| (idl_ix, decoded)))
}

pub fn match_and_collect(
    tx: &Transaction,
    ix: &Instruction,
    idl: &PumpIdl,
    label: String,
    ix_index: usize,
    is_inner: bool,
) -> Result<Option<InstructionContext>, Box<dyn std::error::Error>> {
    if let Some((idl_ix, _decoded)) = match_instruction(ix, idl)? {
        let mapped_accounts = map_ix_accounts(tx, ix, idl_ix);

        let mut accounts_map = BTreeMap::new();
        for (name, pk) in mapped_accounts {
            accounts_map.insert(name, pk);
        }

        Ok(Some(InstructionContext {
            ix_name: idl_ix.name.clone(),
            label,
            ix_index,
            is_inner,
            accounts: accounts_map,
        }))
    } else {
        Ok(None)
    }
}

/// Collect InstructionContext for outer+inner arrays already filtered by program
pub fn collect_contexts<'a>(
    tx: &'a Transaction,
    idl: &PumpIdl,
    outer_instructions: &[&'a Instruction],
    inner_instructions: &[&'a Instruction],
    tx_idx: usize,
) -> Result<Vec<InstructionContext>, Box<dyn std::error::Error>> {
    let mut out: Vec<InstructionContext> = Vec::new();

    for (idx, ix) in outer_instructions.iter().enumerate() {
        if let Some(ctx) = match_and_collect(
            tx,
            ix,
            idl,
            format!("tx #{tx_idx} outer #{idx}"),
            idx,
            false,
        )? {
            out.push(ctx);
        }
    }

    for (idx, ix) in inner_instructions.iter().enumerate() {
        if let Some(ctx) = match_and_collect(
            tx,
            ix,
            idl,
            format!("tx #{tx_idx} inner #{idx}"),
            idx,
            true,
        )? {
            out.push(ctx);
        }
    }

    Ok(out)
}
