use borsh::{BorshDeserialize, BorshSerialize};
use crate::error::*;
/// Program state handler.
use crate::state::*;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    // declare_id,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::{clock::Clock, Sysvar},
};
use spl_associated_token_account::instruction as associated_token_account_instruction;
use std::str::FromStr;

const LOWEST_MINT_LAMPORTS: u64 = 1000;
const CLAIM_SEED_PREFIX: &str = "octo-claim";
const PROJECT_PROGRAM_ID: &str = "FP16xDjSoAcS4NYHNLvpgSFbgUvkNfHbSm8Fo3a9RgxG";

pub struct Processor {}
impl Processor {
    pub fn create_mint_account(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
        bump: u8,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let user_account = next_account_info(accounts_iter)?;
        let clock_account = next_account_info(accounts_iter)?;
        let mint_account = next_account_info(accounts_iter)?;
        let project_account = next_account_info(accounts_iter)?;
        let seller_account = next_account_info(accounts_iter)?;
        let system_program = next_account_info(accounts_iter)?;

        if *project_account.key != Pubkey::from_str(PROJECT_PROGRAM_ID).unwrap_or_default() {
            return Err(LaunchTokenError::IncorrectProject.into());
        }

        let project_data = project_account.try_borrow_mut_data()?;
        let project_info = ProjectInfo::try_from_slice(&project_data)?;
        let clock = Clock::from_account_info(&clock_account)?;
        if clock.unix_timestamp < project_info.launch_start_time as i64 {
            return Err(LaunchTokenError::LaunchNotStart.into());
        }

        if clock.unix_timestamp > project_info.launch_end_time as i64 {
            return Err(LaunchTokenError::LaunchEnd.into());
        }

        if *seller_account.key != project_info.seller_account {
            return Err(LaunchTokenError::IncorrectSellerAddress.into());
        }

        if amount < LOWEST_MINT_LAMPORTS {
            msg!("input amount is less than limit");
            return Err(LaunchTokenError::LessThanLowestAmount.into());
        }

        if user_account.lamports() < amount {
            return Err(ProgramError::InsufficientFunds);
        }

        if mint_account.lamports() == 0 {
            msg!("Need create new pda account");
            let lamports_required = (Rent::get()?).minimum_balance(MintInfo::SIZE);
            invoke_signed(
                &system_instruction::create_account(
                    user_account.key,
                    mint_account.key,
                    lamports_required,
                    MintInfo::SIZE as u64,
                    program_id,
                ),
                &[
                    user_account.clone(),
                    mint_account.clone(),
                    system_program.clone(),
                ],
                &[&[
                    MintInfo::SEED_PREFIX.as_bytes(),
                    user_account.key.as_ref(),
                    &[bump],
                ]],
            )?;
        } else {
            msg!("Pda account already exist,need update");
        }

        let mut mint_info = MintInfo::try_from_slice(&mint_account.try_borrow_mut_data()?)?;
        msg!("Before: mint account amount is {} ", mint_info.amount);

        if *mint_account.owner != *program_id {
            return Err(ProgramError::IncorrectProgramId);
        }
        mint_info.buyer = *user_account.key;
        mint_info.amount += amount;
        mint_info.serialize(&mut *mint_account.data.borrow_mut())?;

        invoke(
            &system_instruction::transfer(user_account.key, &project_info.seller_account, amount),
            &[
                user_account.clone(),
                seller_account.clone(),
                system_program.clone(),
            ],
        )?;

        msg!("Mint user = {},amount = {}", user_account.key, amount);
        msg!("After:mint account amount is {} ", mint_info.amount);

        Ok(())
    }

    pub fn create_claim_account(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        bump: u8,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let user_account = next_account_info(accounts_iter)?;
        let claim_account = next_account_info(accounts_iter)?;
        let project_account = next_account_info(accounts_iter)?;
        let system_program = next_account_info(accounts_iter)?;

        if *project_account.key != Pubkey::from_str(PROJECT_PROGRAM_ID).unwrap_or_default() {
            return Err(LaunchTokenError::IncorrectProject.into());
        }

        let project_info = ProjectInfo::try_from_slice(&project_account.try_borrow_mut_data()?)?;
        let seller = project_info.seller_account;
        let lamports_required = (Rent::get()?).minimum_balance(0);
        invoke_signed(
            &system_instruction::create_account(
                user_account.key,
                claim_account.key,
                lamports_required,
                0u64,
                program_id,
            ),
            &[
                user_account.clone(),
                claim_account.clone(),
                system_program.clone(),
            ],
            &[&[CLAIM_SEED_PREFIX.as_bytes(), seller.as_ref(), &[bump]]],
        )?;

        msg!("Claim pda account is created.");
        Ok(())
    }

    pub fn process_claim(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let user_account = next_account_info(accounts_iter)?;
        let clock_account = next_account_info(accounts_iter)?;
        let mint_account = next_account_info(accounts_iter)?;
        let token_account = next_account_info(accounts_iter)?;
        let project_account = next_account_info(accounts_iter)?;
        let claim_pda_account = next_account_info(accounts_iter)?;
        let from_ata_account = next_account_info(accounts_iter)?;
        let to_ata_account = next_account_info(accounts_iter)?;
        let token_program = next_account_info(accounts_iter)?;
        let system_program = next_account_info(accounts_iter)?;
        let associated_token_program = next_account_info(accounts_iter)?;

        if *project_account.key != Pubkey::from_str(PROJECT_PROGRAM_ID).unwrap_or_default() {
            return Err(LaunchTokenError::IncorrectProject.into());
        }

        if mint_account.lamports() == 0 {
            return Err(LaunchTokenError::MintAccountNotExist.into());
        }

        if *mint_account.owner != *program_id {
            return Err(ProgramError::IncorrectProgramId);
        }

        let project_info = ProjectInfo::try_from_slice(&project_account.try_borrow_mut_data()?)?;
        let clock = Clock::from_account_info(&clock_account)?;
        // if clock.unix_timestamp < project_info.claim_start_time as i64 {
        //     return Err(LaunchTokenError::ClaimNotStart.into());
        // }

        let mut mint_info = MintInfo::try_from_slice(&mint_account.try_borrow_mut_data()?)?;
        if mint_info.buyer != *user_account.key {
            return Err(LaunchTokenError::PermissionForbidden.into());
        }
        if mint_info.amount <= 0 || mint_info.is_claimed {
            return Err(LaunchTokenError::NoReserveToClaim.into());
        }
        let token_price = project_info.token_price;
        // (buyed_sol_amount/10^8)*price*10^9
        let token_amount = mint_info.amount * token_price * 10;
        msg!(
            "user {} buyed {}sol at the price {},should get {} token",
            user_account.key,
            mint_info.amount,
            token_price,
            token_amount
        );

        let bump = 255;
        let authority_seeds = &[
            CLAIM_SEED_PREFIX.as_bytes(),
            &project_info.seller_account.as_ref(),
            &[bump],
        ];
        let signers = &[&authority_seeds[..]];
        if to_ata_account.lamports() == 0 {
            msg!("Creating associated token account for recipient...");
            invoke(
                &associated_token_account_instruction::create_associated_token_account(
                    user_account.key,
                    user_account.key,
                    token_account.key,
                    token_program.key,
                ),
                &[
                    token_account.clone(),
                    to_ata_account.clone(),
                    user_account.clone(),
                    user_account.clone(),
                    system_program.clone(),
                    token_program.clone(),
                    associated_token_program.clone(),
                ],
            )?;
        } else {
            msg!("Associated token account exists.");
        }

        let ix = spl_token::instruction::transfer(
            &spl_token::ID,
            &from_ata_account.key,
            &to_ata_account.key,
            &claim_pda_account.key,
            &[claim_pda_account.key, user_account.key],
            token_amount,
        )?;

        invoke_signed(
            &ix,
            &[
                token_account.clone(),
                from_ata_account.clone(),
                to_ata_account.clone(),
                claim_pda_account.clone(),
                user_account.clone(),
                token_program.clone(),
            ],
            signers,
        );
        mint_info.is_claimed = true;
        mint_info.serialize(&mut *mint_account.data.borrow_mut())?;
        Ok(())
    }
}
