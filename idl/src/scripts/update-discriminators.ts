import * as idl from "../target/idl/flipcash.json";

// Pulled from:
// flipcash-program/api/src/instruction.rs - InstructionType enum
const instructionValues: Record<string, number[]> = {
    initialize_currency: [1],
    initialize_pool: [2],
    initialize_metadata: [3],
    buy_tokens: [4],
    sell_tokens: [5],
    buy_and_deposit_into_vm: [6],
    sell_and_deposit_into_vm: [7],
    burn_fees: [8],
};

// Pulled from:
// flipcash-program/api/src/state/mod.rs - AccountType enum
const accountValues: Record<string, number[]> = {
    CurrencyConfig: [1, 0, 0, 0, 0, 0, 0, 0],
    LiquidityPool: [2, 0, 0, 0, 0, 0, 0, 0],
};

function updateDiscriminators() {
    const instructions = (idl as any).instructions;
    for (let ix of instructions) {
        const val = instructionValues[ix.name];
        if (val === undefined) {
            throw new Error(`Instruction ${ix.name} not found`);
        }
        ix.discriminator = val;
    }

    const accounts = (idl as any).accounts;
    for (const acc of accounts) {
        const val = accountValues[acc.name];
        if (val === undefined) {
            throw new Error(`Account ${acc.name} not found`);
        }
        acc.discriminator = val;
    }

    return idl;
}

console.log(JSON.stringify(updateDiscriminators(), null, 2));
