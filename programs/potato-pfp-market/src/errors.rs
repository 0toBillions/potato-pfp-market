use anchor_lang::prelude::*;

#[error_code]
pub enum PotatoError {
    #[msg("Image URL too long (max 200 characters)")]
    UrlTooLong,
    
    #[msg("Market is not currently active")]
    MarketNotActive,
    
    #[msg("This round has already ended")]
    RoundEnded,
    
    #[msg("Arithmetic overflow")]
    MathOverflow,
    
    #[msg("Stake amount is below minimum")]
    StakeTooSmall,
    
    #[msg("Winner has already been picked for this round")]
    WinnerAlreadyPicked,
    
    #[msg("No submissions in this round")]
    NoSubmissions,
    
    #[msg("Rewards already claimed")]
    AlreadyClaimed,
    
    #[msg("No winner has been picked yet")]
    NoWinnerYet,
    
    #[msg("This is not the winning submission")]
    NotWinningSubmission,
    
    #[msg("Your stake is not on the winning submission")]
    StakeNotOnWinner,
    
    #[msg("Round has not ended yet")]
    RoundNotEnded,
}
