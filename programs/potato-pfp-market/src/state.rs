use anchor_lang::prelude::*;

/// Global market state - one per deployment
#[account]
#[derive(InitSpace)]
pub struct MarketState {
    /// Admin who can pick winners (the AI agent wallet)
    pub admin: Pubkey,
    /// $POTATO token mint address
    pub token_mint: Pubkey,
    /// Vault holding staked tokens
    pub vault: Pubkey,
    /// Fee in lamports to submit an image
    pub submission_fee: u64,
    /// Minimum stake amount
    pub min_stake: u64,
    /// Current round number
    pub round: u64,
    /// Number of submissions this round
    pub submissions_count: u64,
    /// Total tokens staked this round
    pub total_staked: u64,
    /// Whether the market is accepting submissions/stakes
    pub is_active: bool,
    /// Winning submission (None until picked)
    pub winner: Option<Pubkey>,
    /// PDA bump seed
    pub bump: u8,
}

/// A submitted potato image
#[account]
#[derive(InitSpace)]
pub struct Submission {
    /// Who submitted this image
    pub submitter: Pubkey,
    /// URL to the image (IPFS, Arweave, etc.)
    #[max_len(200)]
    pub image_url: String,
    /// SHA256 hash of the image for verification
    pub image_hash: [u8; 32],
    /// Total tokens staked on this submission
    pub total_staked: u64,
    /// Number of unique stakers
    pub stakers_count: u64,
    /// Round this submission belongs to
    pub round: u64,
    /// Index within the round
    pub index: u64,
    /// PDA bump seed
    pub bump: u8,
}

/// A user's stake on a submission
#[account]
#[derive(InitSpace)]
pub struct StakeAccount {
    /// Who made this stake
    pub staker: Pubkey,
    /// Which submission they staked on
    pub submission: Pubkey,
    /// Amount staked
    pub amount: u64,
    /// Round this stake was made in
    pub round: u64,
    /// Whether rewards have been claimed
    pub claimed: bool,
    /// PDA bump seed
    pub bump: u8,
}
