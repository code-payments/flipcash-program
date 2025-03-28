#![cfg(test)]
use flipcash_api::prelude::InstructionType;
use litesvm::types::TransactionMetadata;
use solana_sdk::transaction::Transaction;
use pretty_hex::*;


pub fn print_tx(meta: TransactionMetadata, tx: Transaction) {
    let msg = tx.message().serialize();

    println!("\n");
    println!("--------------------------------------------------------------------------------");
    println!("sig:\t{:?}", meta.signature);
    println!("len:\t{:?}", msg.len());

    for i in 0..tx.message.instructions.len() {
        let ix = &tx.message.instructions[i];
        let ix_type = InstructionType::try_from(ix.data[0] as u8).unwrap();

        println!("\nix:\t{:?} ({})", ix_type, ix.data[0]);
        println!("accounts:");

        for key in &ix.accounts {
            println!("\t{}: {:?}", key, tx.message.account_keys[*key as usize]);
        }

        println!("\ndata:\n\t{:?}", ix.data);
        println!("\n\n{}\n", pretty_hex(&ix.data))
    }

    println!("");
    println!("cu:\t{:?}", meta.compute_units_consumed);
    println!("logs:");
    for log in &meta.logs {
        println!("\t{:?}", log);
    }
    println!("");
}
