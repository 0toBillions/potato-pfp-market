use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

pub mod state;
pub mod errors;

use state::*;
use errors::*;

#[program]
pub mod potato_pfp_market {
    use super::*;

    /// Initialize the PFP prediction market
    /// Only admin (the AI agent) can call this
    pub fn initialize_market(
        ctx: Context<InitializeMarket>,
        submission_fee: u64,      // Fee in lamports to submit an image
        min_stake: u64,           // Minimum stake amount in tokens
    ) -> Result<()> {
        let market = &mut ctx.accounts.market;
        
        market.admin = ctx.accounts.admin.key();
        market.token_mint = ctx.accounts.token_mint.key();
        market.vault = ctx.accounts.vault.key();
        market.submission_fee = submission_fee;
        market.min_stake = min_stake;
        market.round = 1;
        market.submissions_count = 0;
        market.total_staked = 0;
        market.is_active = true;
        market.winner = None;
        market.bump = ctx.bumps.market;
        
        msg!("🥔 Potato PFP Market initialized! Round 1 active.");
        Ok(())
    }

    /// Submit a potato image to the competition
    pub fn submit_image(
        ctx: Context<SubmitImage>,
        image_url: String,
        image_hash: [u8; 32],  // SHA256 hash for verification
    ) -> Result<()> {
        require!(image_url.len() <= 200, PotatoError::UrlTooLong);
        require!(ctx.accounts.market.is_active, PotatoError::MarketNotActive);
        require!(ctx.accounts.market.winner.is_none(), PotatoError::RoundEnded);
        
        let market = &mut ctx.accounts.market;
        let submission = &mut ctx.accounts.submission;
        
        // Transfer submission fee to admin
        if market.submission_fee > 0 {
            let cpi_context = CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.submitter.to_account_info(),
                    to: ctx.accounts.admin.to_account_info(),
                },
            );
            anchor_lang::system_program::transfer(cpi_context, market.submission_fee)?;
        }
        
        submission.submitter = ctx.accounts.submitter.key();
        submission.image_url = image_url.clone();
        submission.image_hash = image_hash;
        submission.total_staked = 0;
        submission.stakers_count = 0;
        submission.round = market.round;
        submission.index = market.submissions_count;
        submission.bump = ctx.bumps.submission;
        
        market.submissions_count = market.submissions_count.checked_add(1)
            .ok_or(PotatoError::MathOverflow)?;
        
        msg!("🥔 New submission #{}: {}", submission.index, image_url);
        Ok(())
    }

    /// Stake $POTATO tokens on a submission
    pub fn stake(
        ctx: Context<StakeTokens>,
        amount: u64,
    ) -> Result<()> {
        let market = &ctx.accounts.market;
        
        require!(market.is_active, PotatoError::MarketNotActive);
        require!(market.winner.is_none(), PotatoError::RoundEnded);
        require!(amount >= market.min_stake, PotatoError::StakeTooSmall);
        
        // Transfer tokens from user to vault
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
            authority: ctx.accounts.staker.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;
        
        // Update stake account
        let stake_account = &mut ctx.accounts.stake_account;
        let is_new_stake = stake_account.amount == 0;
        
        stake_account.staker = ctx.accounts.staker.key();
        stake_account.submission = ctx.accounts.submission.key();
        stake_account.amount = stake_account.amount.checked_add(amount)
            .ok_or(PotatoError::MathOverflow)?;
        stake_account.round = market.round;
        stake_account.claimed = false;
        stake_account.bump = ctx.bumps.stake_account;
        
        // Update submission totals
        let submission = &mut ctx.accounts.submission;
        submission.total_staked = submission.total_staked.checked_add(amount)
            .ok_or(PotatoError::MathOverflow)?;
        if is_new_stake {
            submission.stakers_count = submission.stakers_count.checked_add(1)
                .ok_or(PotatoError::MathOverflow)?;
        }
        
        // Update market totals
        let market = &mut ctx.accounts.market;
        market.total_staked = market.total_staked.checked_add(amount)
            .ok_or(PotatoError::MathOverflow)?;
        
        msg!("🥔 Staked {} tokens on submission #{}", amount, submission.index);
        Ok(())
    }

    /// Admin picks the winning submission
    /// Only the AI agent (admin) can call this
    pub fn pick_winner(
        ctx: Context<PickWinner>,
    ) -> Result<()> {
        let market = &mut ctx.accounts.market;
        
        require!(market.is_active, PotatoError::MarketNotActive);
        require!(market.winner.is_none(), PotatoError::WinnerAlreadyPicked);
        require!(market.submissions_count > 0, PotatoError::NoSubmissions);
        
        // Set the winner
        market.winner = Some(ctx.accounts.winning_submission.key());
        market.is_active = false;
        
        let submission = &ctx.accounts.winning_submission;
        msg!("🥔🏆 WINNER PICKED! Submission #{} by {} wins!", 
            submission.index, 
            submission.submitter
        );
        msg!("🥔 Image: {}", submission.image_url);
        msg!("🥔 Total staked on winner: {}", submission.total_staked);
        
        Ok(())
    }

    /// Claim rewards - winners get proportional share of total pot
    pub fn claim(
        ctx: Context<ClaimRewards>,
    ) -> Result<()> {
        let market = &ctx.accounts.market;
        let stake_account = &ctx.accounts.stake_account;
        let winning_submission = &ctx.accounts.winning_submission;
        
        require!(!stake_account.claimed, PotatoError::AlreadyClaimed);
        require!(market.winner.is_some(), PotatoError::NoWinnerYet);
        require!(
            market.winner.unwrap() == winning_submission.key(),
            PotatoError::NotWinningSubmission
        );
        require!(
            stake_account.submission == winning_submission.key(),
            PotatoError::StakeNotOnWinner
        );
        
        // Calculate reward: (user_stake / winner_total_staked) * total_pot
        let user_stake = stake_account.amount;
        let winner_total = winning_submission.total_staked;
        let total_pot = market.total_staked;
        
        // reward = (user_stake * total_pot) / winner_total
        let reward = (user_stake as u128)
            .checked_mul(total_pot as u128)
            .ok_or(PotatoError::MathOverflow)?
            .checked_div(winner_total as u128)
            .ok_or(PotatoError::MathOverflow)? as u64;
        
        // Transfer tokens from vault to user
        let market_key = ctx.accounts.market.key();
        let seeds = &[
            b"market".as_ref(),
            market_key.as_ref(),
            &[market.bump],
        ];
        let signer_seeds = &[&seeds[..]];
        
        // Actually use market PDA seeds
        let market_seeds = &[b"market".as_ref(), &[market.bump]];
        let market_signer = &[&market_seeds[..]];
        
        let cpi_accounts = Transfer {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.market.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, market_signer);
        token::transfer(cpi_ctx, reward)?;
        
        // Mark as claimed
        let stake_account = &mut ctx.accounts.stake_account;
        stake_account.claimed = true;
        
        msg!("🥔💰 Claimed {} tokens!", reward);
        Ok(())
    }

    /// Start a new round (admin only)
    pub fn new_round(
        ctx: Context<NewRound>,
    ) -> Result<()> {
        let market = &mut ctx.accounts.market;
        
        require!(market.winner.is_some(), PotatoError::RoundNotEnded);
        
        market.round = market.round.checked_add(1)
            .ok_or(PotatoError::MathOverflow)?;
        market.submissions_count = 0;
        market.total_staked = 0;
        market.is_active = true;
        market.winner = None;
        
        msg!("🥔 New round started! Round #{}", market.round);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeMarket<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    
    #[account(
        init,
        payer = admin,
        space = 8 + MarketState::INIT_SPACE,
        seeds = [b"market"],
        bump,
    )]
    pub market: Account<'info, MarketState>,
    
    /// The $POTATO token mint
    pub token_mint: Account<'info, anchor_spl::token::Mint>,
    
    /// Vault to hold staked tokens (ATA of market PDA)
    #[account(
        init,
        payer = admin,
        token::mint = token_mint,
        token::authority = market,
        seeds = [b"vault", market.key().as_ref()],
        bump,
    )]
    pub vault: Account<'info, TokenAccount>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(image_url: String, image_hash: [u8; 32])]
pub struct SubmitImage<'info> {
    #[account(mut)]
    pub submitter: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"market"],
        bump = market.bump,
    )]
    pub market: Account<'info, MarketState>,
    
    /// CHECK: Admin receives submission fee
    #[account(mut, address = market.admin)]
    pub admin: AccountInfo<'info>,
    
    #[account(
        init,
        payer = submitter,
        space = 8 + Submission::INIT_SPACE,
        seeds = [b"submission", market.key().as_ref(), &market.submissions_count.to_le_bytes()],
        bump,
    )]
    pub submission: Account<'info, Submission>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct StakeTokens<'info> {
    #[account(mut)]
    pub staker: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"market"],
        bump = market.bump,
    )]
    pub market: Account<'info, MarketState>,
    
    #[account(
        mut,
        seeds = [b"submission", market.key().as_ref(), &submission.index.to_le_bytes()],
        bump = submission.bump,
    )]
    pub submission: Account<'info, Submission>,
    
    #[account(
        init_if_needed,
        payer = staker,
        space = 8 + StakeAccount::INIT_SPACE,
        seeds = [b"stake", submission.key().as_ref(), staker.key().as_ref()],
        bump,
    )]
    pub stake_account: Account<'info, StakeAccount>,
    
    #[account(
        mut,
        constraint = user_token_account.owner == staker.key(),
        constraint = user_token_account.mint == market.token_mint,
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [b"vault", market.key().as_ref()],
        bump,
    )]
    pub vault: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PickWinner<'info> {
    #[account(address = market.admin)]
    pub admin: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"market"],
        bump = market.bump,
    )]
    pub market: Account<'info, MarketState>,
    
    #[account(
        seeds = [b"submission", market.key().as_ref(), &winning_submission.index.to_le_bytes()],
        bump = winning_submission.bump,
        constraint = winning_submission.round == market.round,
    )]
    pub winning_submission: Account<'info, Submission>,
}

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(mut)]
    pub claimer: Signer<'info>,
    
    #[account(
        seeds = [b"market"],
        bump = market.bump,
    )]
    pub market: Account<'info, MarketState>,
    
    #[account(
        constraint = market.winner == Some(winning_submission.key()),
    )]
    pub winning_submission: Account<'info, Submission>,
    
    #[account(
        mut,
        seeds = [b"stake", winning_submission.key().as_ref(), claimer.key().as_ref()],
        bump = stake_account.bump,
        constraint = stake_account.staker == claimer.key(),
    )]
    pub stake_account: Account<'info, StakeAccount>,
    
    #[account(
        mut,
        constraint = user_token_account.owner == claimer.key(),
        constraint = user_token_account.mint == market.token_mint,
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [b"vault", market.key().as_ref()],
        bump,
    )]
    pub vault: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct NewRound<'info> {
    #[account(address = market.admin)]
    pub admin: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"market"],
        bump = market.bump,
    )]
    pub market: Account<'info, MarketState>,
}
