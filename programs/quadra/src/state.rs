use anchor_lang::prelude::*;
use solana_program::{ pubkey, pubkey::Pubkey };

use crate::error;

pub const OWNER_1: Pubkey = pubkey!("d4JmjH2es6fY6znhHiL34fva6VibbJFT6ym6tLR13Pk");
pub const OWNER_2: Pubkey = pubkey!("7yvFUSGY5ueMy9K7ihoDuKpbnAbkXsTgEZe7hVooEMN8");

pub const FEE_SEED: &[u8] = b"fee";
pub const ADMIN_SEED: &[u8] = b"admin";
pub const MEMBERSHIP_SEED: &[u8] = b"membership";
pub const TREASURY_VAULT_SEED: &[u8] = b"treasury_vault";
pub const BURN_SEED: &[u8] = b"burn";
pub const USDC_VAULT_SEED: &[u8] = b"usdc_vault";
pub const STABLE_VAULT_SEED: &[u8] = b"stable_vault";
pub const GOVERNANCE_VAULT_SEED: &[u8] = b"governance_vault";
pub const BENEFICIARY_SEED: &[u8] = b"beneficiary";

pub const MIN_TITLE_LENGTH: usize = 10;
pub const MAX_TITLE_LENGTH: usize = 50;
pub const MAX_DESCRIPTION_LENGTH: usize = 500;

pub const MIN_NAME_LENGTH: usize = 2;
pub const MAX_NAME_LENGTH: usize = 50;
pub const MAX_IMAGE_LENGTH: usize = 500;
pub const MAX_BIO_LENGTH: usize = 500;

pub const DAO_SEED: &[u8] = b"dao";

pub const PROPOSAL_SEED: &[u8] = b"proposal";
pub const USER_PROPOSAL_VOTES_SEED: &[u8] = b"user_proposal_votes";

pub const POOL_INFO_SEED: &[u8] = b"pool_info";
pub const DEPOSIT_FEE: f64 = 0.01;

#[account]
pub struct FeeAccount {
    pub fee_amount: u64,
}

#[account]
pub struct DAO {
    pub creator: Pubkey,
    pub name: String,
    pub image: String,
    pub treasury_vault: Pubkey,
    pub burn_vault: Pubkey,
    pub total_proposals: u64,
    pub min_yes_votes: u64, // minimum yes votes required for a proposal to pass
    pub proposal_creation_fee: u64,
    pub membership_fee: u64,
}

#[account]
pub struct Admin {
    pub dao: Pubkey,
    pub admin: Pubkey,
}

#[account]
pub struct Membership {
    pub dao: Pubkey,
    pub user: Pubkey,
    pub joined_date: i64,
    pub active: bool,
}

#[account]
pub struct Proposal {
    pub creator: Pubkey,
    pub beneficiary: Pubkey,
    pub dao: Pubkey,
    pub title: String,
    pub description: String,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub token_amount: u64,
    pub status: u8, // 0 = active, 1 = passed, 2 = failed
    pub action: u8, // 0 = burn, 1 = transfer
    pub end_date: i64,
    pub executed: bool,
    pub burn_on_vote: bool,
}

#[account]
pub struct UserProposalVotes {
    pub amount: u64,
}

impl DAO {
    // Checks if the provided pubkey is an admin of the DAO
    // pub fn is_admin(&self, pubkey: &Pubkey) -> Result<()> {
    //     match self.admins.contains(pubkey) {
    //         true => Ok(()),
    //         false => Err(error::ErrorCode::NotAuthorized.into()),
    //     }
    // }

    // pub fn is_member(&self, pubkey: &Pubkey) -> Result<()> {
    //     match self.members.contains(pubkey) {
    //         true => Ok(()),
    //         false => Err(error::ErrorCode::NotAuthorized.into()),
    //     }
    // }

    // check length of name, bio, and avatar
    pub fn check_length(&self, name: &str, image: &str) -> Result<()> {
        if name.chars().count() < MIN_NAME_LENGTH {
            return Err(error::ErrorCode::NameTooShort.into());
        }

        if name.chars().count() > MAX_NAME_LENGTH {
            return Err(error::ErrorCode::NameTooLong.into());
        }

        if image.chars().count() > MAX_IMAGE_LENGTH {
            return Err(error::ErrorCode::ImageTooLong.into());
        }

        Ok(())
    }
}

impl Proposal {
    // check length of title and description
    pub fn check_length(&self, title: &str, description: &str) -> Result<()> {
        if title.chars().count() > MAX_TITLE_LENGTH {
            return Err(error::ErrorCode::NameTooLong.into());
        }

        if title.chars().count() < MIN_TITLE_LENGTH {
            return Err(error::ErrorCode::NameTooShort.into());
        }

        if description.chars().count() > MAX_DESCRIPTION_LENGTH {
            return Err(error::ErrorCode::DescriptionTooLong.into());
        }

        Ok(())
    }
}
