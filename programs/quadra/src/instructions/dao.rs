use anchor_lang::prelude::*;
use anchor_spl::token::{ Mint, Token, TokenAccount, Transfer, transfer };
use anchor_spl::associated_token::AssociatedToken;
use anchor_lang::system_program;
use solana_program::{ pubkey, pubkey::Pubkey };
use solana_program::clock::Clock;

use crate::error;
use crate::state::{
    Membership,
    Admin,
    DAO,
    FeeAccount,
    BURN_SEED,
    DAO_SEED,
    FEE_SEED,
    MAX_IMAGE_LENGTH,
    MAX_NAME_LENGTH,
    MEMBERSHIP_SEED,
    TREASURY_VAULT_SEED,
    ADMIN_SEED,
};

pub fn create_dao(
    ctx: Context<CreateDAO>,
    name: String,
    image: String,
    min_yes_votes: u64,
    proposal_creation_fee: u64,
    membership_fee: u64
) -> Result<()> {
    if membership_fee < 1 {
        return Err(error::ErrorCode::InvalidFee.into());
    }

    if proposal_creation_fee < 1 {
        return Err(error::ErrorCode::InvalidFee.into());
    }

    let dao = &mut ctx.accounts.dao;
    let fee_account = &ctx.accounts.fee_account;

    dao.check_length(&name, &image)?;

    dao.creator = *ctx.accounts.user.to_account_info().key;
    dao.name = name;
    dao.image = image;
    dao.treasury_vault = *ctx.accounts.treasury_vault.to_account_info().key;
    dao.total_proposals = 0;
    dao.burn_vault = *ctx.accounts.burn_vault.to_account_info().key;
    dao.min_yes_votes = min_yes_votes;
    dao.proposal_creation_fee = proposal_creation_fee;
    dao.membership_fee = membership_fee;

    // send the creation fee to the fee address
    let cpi_context = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        system_program::Transfer {
            from: ctx.accounts.user.to_account_info(),
            to: fee_account.to_account_info(),
        }
    );
    system_program::transfer(cpi_context, fee_account.fee_amount)?;

    Ok(())
}

pub fn join_dao(ctx: Context<JoinDAO>) -> Result<()> {
    let dao = &mut ctx.accounts.dao;
    let membership = &mut ctx.accounts.membership;

    membership.dao = *dao.to_account_info().key;
    membership.user = *ctx.accounts.user.to_account_info().key;
    membership.joined_date = Clock::get()?.unix_timestamp;
    membership.active = true;

    // Transfer the join fee from the user's account to the dao's treasury vault
    let cpi_context = CpiContext::new(ctx.accounts.token_program.to_account_info(), Transfer {
        from: ctx.accounts.user_token_mint_account.to_account_info(),
        to: ctx.accounts.treasury_vault.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    });
    transfer(cpi_context, dao.membership_fee)?;

    Ok(())
}

#[derive(Accounts)]
pub struct CreateDAO<'info> {
    #[account(
        init,
        payer = user,
        seeds = [DAO_SEED, user.key().as_ref()],
        bump,
        space = 8 + std::mem::size_of::<DAO>() + MAX_NAME_LENGTH + MAX_IMAGE_LENGTH
    )]
    pub dao: Box<Account<'info, DAO>>,

    #[account(
        init,
        seeds = [TREASURY_VAULT_SEED, dao.key().as_ref()],
        bump,
        payer = user,
        token::mint = token_mint,
        token::authority = treasury_vault
    )]
    pub treasury_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        seeds = [BURN_SEED, dao.key().as_ref()],
        bump,
        payer = user,
        token::mint = token_mint,
        token::authority = burn_vault
    )]
    pub burn_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [FEE_SEED],
        bump,
    )]
    pub fee_account: Box<Account<'info, FeeAccount>>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct JoinDAO<'info> {
    #[account(mut)]
    pub dao: Box<Account<'info, DAO>>,

    #[account(
        init,
        payer = user,
        space = 8 + std::mem::size_of::<Membership>(),
        seeds = [MEMBERSHIP_SEED, dao.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub membership: Box<Account<'info, Membership>>,

    #[account(
        mut,
        seeds = [TREASURY_VAULT_SEED, dao.key().as_ref()],
        bump,
    )]
    pub treasury_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = user
    )]
    pub user_token_mint_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
