pub mod state;
pub mod error;
pub mod instructions;

use anchor_lang::prelude::*;
use instructions::*;

declare_id!("25Kw1yUstwo9dBugYc3GNY1cniMMwZjatXQWuBfLV2Da");

#[program]
pub mod quadra {
    use super::*;

    pub fn initialize_fee_account(ctx: Context<InitializeFeeAccount>) -> Result<()> {
        instructions::initialize_fee_account(ctx)
    }

    pub fn create_dao(
        ctx: Context<CreateDAO>,
        name: String,
        image: String,
        min_yes_votes: u64,
        proposal_creation_fee: u64,
        membership_fee: u64
    ) -> Result<()> {
        instructions::create_dao(
            ctx,
            name,
            image,
            min_yes_votes,
            proposal_creation_fee,
            membership_fee
        )
    }

    pub fn join_dao(ctx: Context<JoinDAO>) -> Result<()> {
        instructions::join_dao(ctx)
    }

    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        token_amount: u64,
        end_date: i64,
        title: String,
        description: String,
        action: u8,
        burn_on_vote: bool
    ) -> Result<()> {
        instructions::create_proposal(
            ctx,
            token_amount,
            end_date,
            title,
            description,
            action,
            burn_on_vote
        )
    }

    pub fn vote_on_proposal(ctx: Context<VoteOnProposal>, amount: u64, side: u8) -> Result<()> {
        instructions::vote_on_proposal(ctx, amount, side)
    }

    pub fn execute_proposal(ctx: Context<ExecuteProposal>) -> Result<()> {
        instructions::execute_proposal(ctx)
    }
}
