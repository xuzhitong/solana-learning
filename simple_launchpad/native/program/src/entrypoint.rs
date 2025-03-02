use borsh::{BorshDeserialize, BorshSerialize};
use crate::instruction::*;
use crate::processor::Processor;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Ok(mint) = Mint::try_from_slice(instruction_data) {
        msg!("Mint");
        return Processor::create_mint_account(program_id, accounts, mint.amount, mint.bump);
    }

    if let Ok(create_claim_account) = CreateClaimAccount::try_from_slice(instruction_data) {
        msg!("CreateClaimAccount");
        return Processor::create_claim_account(program_id, accounts, create_claim_account.bump);
    }

    if let Ok(_) = Claim::try_from_slice(instruction_data) {
        msg!("Claim");
        return Processor::process_claim(program_id, accounts);
    }

    Err(ProgramError::InvalidInstructionData)
}
