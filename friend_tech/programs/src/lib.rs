use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use solana_program::{msg, program::invoke, program::invoke_signed, system_instruction};

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("1111111111111111111111111111111111");

pub const LAMPORTS_ONE_SOL: u64 = 1_000_000_000;
fn get_price(supply: u64, amount: u64) -> u64 {
    let sum1 = if supply == 0 {
        0
    } else {
        (supply - 1) * (supply) * (2 * (supply - 1) + 1) / 6
    };
    let sum2 = if (supply == 0 && amount == 1) {
        0
    } else {
        (supply - 1 + amount) * (supply + amount) * (2 * (supply - 1 + amount) + 1) / 6
    };
    (sum2 - sum1) * LAMPORTS_ONE_SOL / 16000
}

fn sol_transfer<'info>(
    system_program: AccountInfo<'info>,
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    let cpi_context = CpiContext::new(system_program, Transfer { from, to });
    transfer(cpi_context, amount)?;
    Ok(())
}

#[program]
mod shares {
    use super::*;
    pub fn create_shares(ctx: Context<CreateShares>) -> Result<()> {
        ctx.accounts.shares_info.supply = 0;
        ctx.accounts.shares_info.shares_subject = *ctx.accounts.subject.key;
        ctx.accounts.shares_info.bump = ctx.bumps.shares_info;
        Ok(())
    }

    pub fn create_shares_balance(ctx: Context<CreateSharesBalance>) -> Result<()> {
        ctx.accounts.shares_balance.balance = 0;
        ctx.accounts.shares_balance.shares_subject = *ctx.accounts.shares_subject.key;
        ctx.accounts.shares_balance.user = *ctx.accounts.user.key;
        ctx.accounts.shares_balance.bump = ctx.bumps.shares_balance;
        Ok(())
    }

    pub fn buy_shares(ctx: Context<BuyShares>, amount: u64) -> Result<()> {
        let protocolFeePercent = 5;
        let subjectFeePercent = 3;
        let shares_subject = &ctx.accounts.shares_subject;
        require!(
            ctx.accounts.shares_info.supply > 0 || shares_subject.key() == ctx.accounts.user.key(),
            ErrorCode::FirstShareOnlyForSubject
        );
        let price = get_price(ctx.accounts.shares_info.supply, amount);
        let protocolFee = price * protocolFeePercent / LAMPORTS_ONE_SOL;
        let subjectFee = price * subjectFeePercent / LAMPORTS_ONE_SOL;
        msg!(
            "price = {},protocolFee={},subjectFee={}",
            price,
            protocolFee,
            subjectFee
        );
        require!(
            ctx.accounts.user.lamports() >= price + protocolFee + subjectFee,
            ErrorCode::InsufficientPayment
        );
        ctx.accounts.shares_balance.balance += amount;
        ctx.accounts.shares_info.supply += amount;
        msg!(
            "user {} buy shares {} amount {} at price {}",
            ctx.accounts.user.key,
            shares_subject.key,
            amount,
            price
        );

        sol_transfer(
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.user.to_account_info(),
            ctx.accounts.vault.to_account_info(),
            price,
        )?;
        sol_transfer(
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.user.to_account_info(),
            ctx.accounts.protocol_fee_destination.to_account_info(),
            protocolFee,
        )?;
        sol_transfer(
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.user.to_account_info(),
            ctx.accounts.shares_subject.to_account_info(),
            subjectFee,
        )?;
        Ok(())
    }

    pub fn sell_shares(ctx: Context<BuyShares>, amount: u64) -> Result<()> {
        let protocolFeePercent = 5;
        let subjectFeePercent = 3;
        let shares_subject = &ctx.accounts.shares_subject;
        require!(
            ctx.accounts.shares_info.supply > amount,
            ErrorCode::SellLastShare
        );
        let price = get_price(ctx.accounts.shares_info.supply - amount, amount);
        let protocolFee = price * protocolFeePercent / LAMPORTS_ONE_SOL;
        let subjectFee = price * subjectFeePercent / LAMPORTS_ONE_SOL;
        msg!(
            "price = {},protocolFee={},subjectFee={}",
            price,
            protocolFee,
            subjectFee
        );
        require!(
            ctx.accounts.shares_balance.balance > amount,
            ErrorCode::InsufficientShares
        );
        require!(
            ctx.accounts.vault.to_account_info().lamports() >= price,
            ErrorCode::InsufficientPayment
        );
        ctx.accounts.shares_balance.balance -= amount;
        ctx.accounts.shares_info.supply -= amount;
        msg!(
            "user {} sell shares {} amount {} at price {}",
            ctx.accounts.user.key,
            shares_subject.key,
            amount,
            price
        );

        **ctx
            .accounts
            .vault
            .to_account_info()
            .try_borrow_mut_lamports()? -= price;
        **ctx
            .accounts
            .user
            .to_account_info()
            .try_borrow_mut_lamports()? += price - protocolFee - subjectFee;
        **ctx
            .accounts
            .protocol_fee_destination
            .to_account_info()
            .try_borrow_mut_lamports()? += protocolFee;
        **ctx
            .accounts
            .shares_subject
            .to_account_info()
            .try_borrow_mut_lamports()? += subjectFee;

        Ok(())
    }

}

#[derive(Accounts)]
pub struct CreateShares<'info> {
    #[account(
        init, 
        payer = subject, 
        space = SharesInfoAccount::SIZE,
        seeds = [b"shares",subject.key().as_ref()],
        bump
    )]
    pub shares_info: Account<'info, SharesInfoAccount>,
    #[account(mut)]
    pub subject: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateSharesBalance<'info> {
    #[account(
        init, 
        payer = user,
        space = SharesBalanceAccount::SIZE,
        seeds = [shares_subject.key().as_ref(),user.key().as_ref()],
        bump
    )]
    pub shares_balance: Account<'info, SharesBalanceAccount>,
    pub shares_subject: AccountInfo<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BuyShares<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init_if_needed,
        payer = user,
        space = 8,
        seeds = [b"vault"],
        bump
    )]
    pub vault: AccountInfo<'info>,
    pub shares_subject: AccountInfo<'info>,
    #[account(
        mut,
        has_one = shares_subject,
        seeds = [b"shares",shares_subject.key().as_ref()],
        bump = shares_info.bump
    )]
    pub shares_info: Account<'info, SharesInfoAccount>,
    #[account(
        mut,
        has_one = shares_subject,
        has_one = user,
        seeds = [shares_subject.key().as_ref(),user.key().as_ref()],
        bump = shares_balance.bump,
    )]
    pub shares_balance: Account<'info, SharesBalanceAccount>,
    pub protocol_fee_destination: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SellShares<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [b"vault"],
        bump
    )]
    pub vault: AccountInfo<'info>,
    pub shares_subject: AccountInfo<'info>,
    #[account(
        mut,
        has_one = shares_subject,
        seeds = [b"shares",shares_subject.key().as_ref()],
        bump = shares_info.bump
    )]
    pub shares_info: Account<'info, SharesInfoAccount>,
    #[account(
        mut,
        has_one = shares_subject,
        has_one = user,
        seeds = [shares_subject.key().as_ref(),user.key().as_ref()],
        bump = shares_balance.bump,
    )]
    pub shares_balance: Account<'info, SharesBalanceAccount>,
    pub protocol_fee_destination: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

// #[account]
// pub struct SharesParameter {

//     protocol_fee_destination
// }
#[account]
pub struct SharesInfoAccount {
    pub shares_subject: Pubkey,
    pub supply: u64,
    pub bump: u8,
}

impl SharesInfoAccount {
    pub const SIZE: usize = 8 + 32 + 8 + 1;
}

#[account]
pub struct SharesBalanceAccount {
    pub shares_subject: Pubkey,
    pub user: Pubkey,
    pub balance: u64,
    pub bump: u8,
}

impl SharesBalanceAccount {
    pub const SIZE: usize = 8 + 32 * 2 + 8 + 1;
}

#[error_code]
pub enum ErrorCode {
    #[msg("Only the shares' subject can buy the first share")]
    FirstShareOnlyForSubject,
    #[msg("Insufficient payment")]
    InsufficientPayment,
    #[msg("Account already exist")]
    AccountAlreadyExist,
    #[msg("Cannot sell the last share")]
    SellLastShare,
    #[msg("Insufficient shares")]
    InsufficientShares,
}
