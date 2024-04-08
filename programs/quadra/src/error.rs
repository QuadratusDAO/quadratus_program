use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Already initialized.")]
    IsInitialized,

    #[msg("Tokens are already staked.")]
    IsStaked,

    #[msg("Tokens are not staked.")]
    NotStaked,

    #[msg("No tokens to stake.")]
    NoTokens,

    #[msg("Invalid amount.")]
    InvalidTokenAmount,

    #[msg("Insufficient funds.")]
    InsufficientFunds,

    #[msg("Invalid destination address.")]
    InvalidDestination,

    #[msg("Insufficient treasury balance.")]
    InsufficientTreasuryBalance,

    #[msg("Insufficient collateral balance.")]
    InsufficientCollateral,

    #[msg("Invalid deposit.")]
    InvalidInitialDeposit,

    #[msg("Invalid minimum lock up period.")]
    InvalidLockUp,

    #[msg("Invalid ending slot.")]
    InvalidEndingSlot,

    #[msg("Invalid vote limit.")]
    InvalidVoteLimit,

    #[msg("Invalid beneficiary.")]
    InvalidBeneficiary,

    #[msg("Lockup period has not ended.")]
    LockupPeriodNotEnded,

    #[msg("Invalid fee.")]
    InvalidFee,

    #[msg("Unauthorized user.")]
    NotAuthorized,

    #[msg("Proposal has already passed.")]
    ProposalAlreadyPassed,

    #[msg("The voting period has ended.")]
    ProposalEnded,

    #[msg("The voting period has not ended.")]
    ProposalActive,

    #[msg("Proposal already executed")]
    ProposalAlreadyExecuted,

    #[msg("Invalid proposal action.")]
    InvalidProposalAction,

    #[msg("Invalid proposal creator.")]
    InvalidProposalCreator,

    #[msg("Admin length must be between 1 and 5.")]
    InvalidAdmins,

    #[msg("DAO and Proposal mismatch.")]
    InvalidProposal,

    #[msg("Name is too long.")]
    NameTooLong,

    #[msg("Name is too short.")]
    NameTooShort,

    #[msg("Bio is too long.")]
    BioTooLong,

    #[msg("Image is too long.")]
    ImageTooLong,

    #[msg("Description is too long.")]
    DescriptionTooLong,

    #[msg("Invalid token mint")]
    InvalidTokenMint,

    #[msg("Invalid fee address")]
    InvalidFeeAddress,

    #[msg("Invalid token account")]
    AlreadyMember,
}
