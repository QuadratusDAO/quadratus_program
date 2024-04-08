use anchor_lang::prelude::*;
use anchor_spl::token::{ transfer, Mint, Token, TokenAccount, Transfer };
use anchor_spl::associated_token::AssociatedToken;
use solana_program::clock::Clock;

use crate::error;
use crate::state::{
    DAO,
    Proposal,
    UserProposalVotes,
    Membership,
    MEMBERSHIP_SEED,
    TREASURY_VAULT_SEED,
    USER_PROPOSAL_VOTES_SEED,
    PROPOSAL_SEED,
    BURN_SEED,
    MAX_TITLE_LENGTH,
    MAX_DESCRIPTION_LENGTH,
    BENEFICIARY_SEED,
};

pub fn create_proposal(
    ctx: Context<CreateProposal>,
    token_amount: u64,
    end_date: i64,
    title: String,
    description: String,
    action: u8,
    burn_on_vote: bool
) -> Result<()> {
    let dao = &mut ctx.accounts.dao;
    let proposal = &mut ctx.accounts.proposal;
    let treasury_vault = &mut ctx.accounts.treasury_vault;
    let beneficary = &ctx.accounts.beneficiary;

    if end_date <= Clock::get().unwrap().unix_timestamp {
        return Err(error::ErrorCode::InvalidEndingSlot.into());
    }

    if token_amount > treasury_vault.amount {
        return Err(error::ErrorCode::InsufficientTreasuryBalance.into());
    }

    if ctx.accounts.user_token_mint_account.mint != ctx.accounts.token_mint.key() {
        return Err(error::ErrorCode::InvalidTokenMint.into());
    }

    proposal.check_length(&title, &description)?;

    proposal.dao = dao.key();
    proposal.creator = *ctx.accounts.user.key;
    proposal.beneficiary = beneficary.key();
    proposal.status = 0;
    proposal.title = title;
    proposal.description = description;
    proposal.yes_votes = 0;
    proposal.no_votes = 0;
    proposal.token_amount = token_amount;
    proposal.action = action;
    proposal.end_date = end_date;
    proposal.executed = false;
    proposal.burn_on_vote = burn_on_vote;

    dao.total_proposals += 1;

    // Transfer the mint token from the user's account to the proposal's treasury vault
    let cpi_context = CpiContext::new(ctx.accounts.token_program.to_account_info(), Transfer {
        from: ctx.accounts.user_token_mint_account.to_account_info(),
        to: ctx.accounts.treasury_vault.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    });

    transfer(cpi_context, dao.proposal_creation_fee)?;

    Ok(())
}

pub fn vote_on_proposal(ctx: Context<VoteOnProposal>, amount: u64, side: u8) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let dao = &ctx.accounts.dao;
    let user_proposal_votes = &mut ctx.accounts.user_proposal_votes;
    let token_mint = &ctx.accounts.token_mint;
    let user = &ctx.accounts.user;

    if proposal.end_date <= Clock::get().unwrap().unix_timestamp {
        return Err(error::ErrorCode::ProposalEnded.into());
    }

    if proposal.dao != dao.key() {
        return Err(error::ErrorCode::InvalidProposal.into());
    }

    // dao.is_member(user.key)?;

    let user_previous_votes = user_proposal_votes.amount;
    let total_votes = amount + user_previous_votes;

    // Calculate the total cost of the votes and multiply by the decimal places of the token mint
    let total_vote_cost = total_votes.pow(2) * (10u64).pow(token_mint.decimals as u32);

    match proposal.burn_on_vote {
        true => {
            // Transfer the mint token from the user's account to the proposal's treasury vault
            let cpi_context = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_mint_account.to_account_info(),
                    to: ctx.accounts.burn_vault.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                }
            );
            transfer(cpi_context, total_vote_cost)?;
        }
        false => {
            // Transfer the mint token from the user's account to the proposal's treasury vault
            let cpi_context = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_mint_account.to_account_info(),
                    to: ctx.accounts.treasury_vault.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                }
            );
            transfer(cpi_context, total_vote_cost)?;
        }
    }

    match side {
        0 => {
            proposal.no_votes += amount;
        }
        1 => {
            proposal.yes_votes += amount;
        }
        _ => {
            return Err(error::ErrorCode::InvalidVoteLimit.into());
        }
    }

    user_proposal_votes.amount += amount;

    Ok(())
}

pub fn execute_proposal(ctx: Context<ExecuteProposal>) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let dao = &ctx.accounts.dao;
    let treasury_vault = &ctx.accounts.treasury_vault;
    let beneficiary_account = &ctx.accounts.beneficiary;

    if proposal.end_date >= Clock::get().unwrap().unix_timestamp {
        return Err(error::ErrorCode::ProposalActive.into());
    }

    if proposal.dao != dao.key() {
        return Err(error::ErrorCode::InvalidProposal.into());
    }

    if proposal.status != 0 {
        return Err(error::ErrorCode::ProposalAlreadyExecuted.into());
    }

    if proposal.yes_votes <= proposal.no_votes {
        proposal.status = 2;

        return Ok(());
    }

    if proposal.yes_votes < dao.min_yes_votes {
        proposal.status = 2;

        return Ok(());
    }

    if beneficiary_account.key() != proposal.beneficiary {
        return Err(error::ErrorCode::InvalidBeneficiary.into());
    }

    match proposal.action {
        0 => {
            // transfer the tokens to the burn address
            let dao_key = dao.key();
            let bump = *ctx.bumps.get("treasury_vault").unwrap();
            let signer: &[&[&[u8]]] = &[&[TREASURY_VAULT_SEED, dao_key.as_ref(), &[bump]]];

            let cpi_context = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: treasury_vault.to_account_info(),
                    to: ctx.accounts.burn_vault.to_account_info(),
                    authority: treasury_vault.to_account_info(),
                },
                signer
            );

            transfer(cpi_context, proposal.token_amount)?;
        }

        1 => {
            // transfer the tokens to the beneficiary
            let dao_key = dao.key();
            let bump = *ctx.bumps.get("treasury_vault").unwrap();
            let signer: &[&[&[u8]]] = &[&[TREASURY_VAULT_SEED, dao_key.as_ref(), &[bump]]];

            let cpi_context = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: treasury_vault.to_account_info(),
                    to: beneficiary_account.to_account_info(),
                    authority: treasury_vault.to_account_info(),
                },
                signer
            );

            transfer(cpi_context, proposal.token_amount)?;
        }

        _ => {
            return Err(error::ErrorCode::InvalidProposalAction.into());
        }
    }

    proposal.status = 1;
    proposal.executed = true;

    Ok(())
}

#[derive(Accounts)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub dao: Box<Account<'info, DAO>>,

    #[account(
        init,
        payer = user,
        space = 8 + std::mem::size_of::<Proposal>() + MAX_TITLE_LENGTH + MAX_DESCRIPTION_LENGTH,
        seeds = [PROPOSAL_SEED, dao.key().as_ref(), dao.total_proposals.to_le_bytes().as_ref()],
        bump
    )]
    pub proposal: Box<Account<'info, Proposal>>,

    #[account(
        mut,
        seeds = [TREASURY_VAULT_SEED, dao.key().as_ref()],
        bump
    )]
    pub treasury_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = token_mint,
        associated_token::authority = beneficiary_owner
    )]
    pub beneficiary: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    /// CHECK: The beneficiary would not match the beneficiary_owner if it were not correct.
    pub beneficiary_owner: AccountInfo<'info>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = user
    )]
    pub user_token_mint_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [MEMBERSHIP_SEED, dao.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub membership: Box<Account<'info, Membership>>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VoteOnProposal<'info> {
    #[account(mut)]
    /// CHECK: It is checked inside the function
    pub dao: Box<Account<'info, DAO>>,

    #[account(mut)]
    /// CHECK: It is checked inside the function
    pub proposal: Box<Account<'info, Proposal>>,

    // stores the users vote count for the given proposal
    #[account(
        init_if_needed,
        seeds = [USER_PROPOSAL_VOTES_SEED, user.key.as_ref(), proposal.key().as_ref()],
        bump,
        space = 8 + std::mem::size_of::<UserProposalVotes>(),
        payer = user
    )]
    pub user_proposal_votes: Box<Account<'info, UserProposalVotes>>,

    #[account(
        mut, 
        seeds = [TREASURY_VAULT_SEED, dao.key().as_ref()],
        bump
    )]
    pub treasury_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut, 
        seeds = [BURN_SEED, dao.key().as_ref()],
        bump
    )]
    pub burn_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = user
    )]
    pub user_token_mint_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [MEMBERSHIP_SEED, dao.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub membership: Box<Account<'info, Membership>>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteProposal<'info> {
    #[account(mut)]
    /// CHECK: It is checked inside the function
    pub dao: Box<Account<'info, DAO>>,

    #[account(mut)]
    /// CHECK: It is checked inside the function
    pub proposal: Box<Account<'info, Proposal>>,

    #[account(
        mut, 
        seeds = [TREASURY_VAULT_SEED, dao.key().as_ref()],
        bump
    )]
    pub treasury_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [BURN_SEED, dao.key().as_ref()],
        bump,
    )]
    pub burn_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = beneficiary_owner
    )]
    pub beneficiary: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    /// CHECK: The beneficiary would not match the beneficiary_owner if it were not correct.
    pub beneficiary_owner: AccountInfo<'info>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
