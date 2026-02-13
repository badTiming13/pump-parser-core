use crate::types::block_notification::{Instruction, Transaction};

/// Collect all accounts: accountKeys + loaded writable + loaded readonly
pub fn get_accounts(tx: &Transaction) -> Vec<&String> {
    let account_keys = &tx.transaction.message.account_keys;
    let loaded_w = &tx.meta.loaded_addresses.writable;
    let loaded_r = &tx.meta.loaded_addresses.readonly;

    account_keys
        .iter()
        .chain(loaded_w.iter())
        .chain(loaded_r.iter())
        .collect()
}

/// find index of program address in allAccounts
pub fn find_program_index(tx: &Transaction, program_address: &str) -> Option<u8> {
    let accounts = get_accounts(tx);

    let pos = accounts
        .iter()
        .position(|address| address.as_str() == program_address)?;

    Some(u8::try_from(pos).ok()?)
}

/// Return:
/// - all outer instructions of that program
/// - all inner instructions of that program
pub fn collect_program_instructions<'a>(
    tx: &'a Transaction,
    program_index: u8,
) -> (Vec<&'a Instruction>, Vec<&'a Instruction>) {
    let outer: Vec<&Instruction> = tx
        .transaction
        .message
        .instructions
        .iter()
        .filter(|ix| ix.program_id_index == program_index)
        .collect();

    let inner: Vec<&Instruction> = tx
        .meta
        .inner_instructions
        .as_ref()
        .map(|groups| {
            groups
                .iter()
                .flat_map(|group| group.instructions.iter())
                .filter(|ix| ix.program_id_index == program_index)
                .collect()
        })
        .unwrap_or_default();

    (outer, inner)
}

/// Convenience: collect program instructions by program address
pub fn collect_instructions<'a>(
    tx: &'a Transaction,
    program_address: &str,
) -> Option<(Vec<&'a Instruction>, Vec<&'a Instruction>)> {
    let program_index = find_program_index(tx, program_address)?;
    Some(collect_program_instructions(tx, program_index))
}
