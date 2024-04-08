use anchor_lang::prelude::*;
use anchor_spl::token::{ Mint, Token, TokenAccount, Transfer, transfer };
use anchor_spl::associated_token::AssociatedToken;
use anchor_lang::system_program;
use solana_program::{ pubkey, pubkey::Pubkey };
use solana_program::clock::Clock;

use crate::error;
use crate::state::{ FeeAccount, FEE_SEED, OWNER_1, OWNER_2 };

pub fn initialize_fee_account(ctx: Context<InitializeFeeAccount>) -> Result<()> {
    let user = &ctx.accounts.user;
    let fee_account = &mut ctx.accounts.fee_account;

    if user.key != &OWNER_1 && user.key != &OWNER_2 {
        return Err(error::ErrorCode::NotAuthorized.into());
    }

    fee_account.fee_amount = 100000000;
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeFeeAccount<'info> {
    #[account(
        init_if_needed,
        seeds = [FEE_SEED],
        bump,
        payer = user,
        space = 8 + std::mem::size_of::<FeeAccount>()
    )]
    pub fee_account: Box<Account<'info, FeeAccount>>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}
