use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use crate::TidesError;

// ============ Season Pass Constants ============
pub const MAX_LEADERBOARD_SIZE: usize = 1000;
pub const TOP_REWARDS_COUNT: usize = 100;

// ============ Season Pass Account Structures ============

#[account]
pub struct SeasonPassState {
    pub admin: Pubkey,
    pub current_season_id: u64,
    pub paused: bool,
}

impl SeasonPassState {
    pub const LEN: usize = 8 + // discriminator
        32 + // admin
        8 + // current_season_id
        1; // paused
}

#[account]
pub struct Season {
    pub season_id: u64,
    pub name: String,
    pub start_time: i64,
    pub end_time: i64,
    pub pass_price: u64, // Price in SOL (lamports)
    pub is_active: bool,
    pub total_passes: u64,
}

impl Season {
    pub fn calculate_size(name_len: usize) -> usize {
        8 + // discriminator
        8 + // season_id
        4 + name_len + // name String
        8 + // start_time
        8 + // end_time
        8 + // pass_price
        1 + // is_active
        8 // total_passes
    }
}

#[account]
pub struct PlayerSeasonStats {
    pub player: Pubkey,
    pub season_id: u64,
    pub total_earnings: u64,
    pub total_spent: u64,
    pub net_value: i64, // earnings - spent (can be negative)
    pub last_update_time: i64,
    pub has_pass: bool,
    pub token_id: Option<u64>, // Season pass NFT token ID
}

impl PlayerSeasonStats {
    pub const LEN: usize = 8 + // discriminator
        32 + // player
        8 + // season_id
        8 + // total_earnings
        8 + // total_spent
        8 + // net_value (i64)
        8 + // last_update_time
        1 + // has_pass
        1 + 8; // token_id Option<u64>
}

#[account]
pub struct SeasonLeaderboard {
    pub season_id: u64,
    pub players: Vec<Pubkey>,
}

impl SeasonLeaderboard {
    pub fn calculate_size(player_count: usize) -> usize {
        8 + // discriminator
        8 + // season_id
        4 + (player_count * 32) // players Vec
    }
}

// ============ Season Pass Module ============

pub mod season_pass {
    use super::*;

    pub fn create_season(
        season: &mut Account<Season>,
        season_id: u64,
        name: String,
        start_time: i64,
        end_time: i64,
        pass_price: u64,
        clock: &Clock,
    ) -> Result<()> {
        require!(start_time > clock.unix_timestamp, TidesError::StartTimeNotInFuture);
        require!(end_time > start_time, TidesError::InvalidTimeRange);
        require!(!name.is_empty(), TidesError::EmptyString);

        season.season_id = season_id;
        season.name = name;
        season.start_time = start_time;
        season.end_time = end_time;
        season.pass_price = pass_price;
        season.is_active = true;
        season.total_passes = 0;

        Ok(())
    }

    pub fn is_season_active(season: &Account<Season>, clock: &Clock) -> Result<bool> {
        if !season.is_active {
            return Ok(false);
        }
        if clock.unix_timestamp < season.start_time {
            return Ok(false);
        }
        if clock.unix_timestamp > season.end_time {
            return Ok(false);
        }
        Ok(true)
    }

    pub fn update_player_stats(
        stats: &mut Account<PlayerSeasonStats>,
        earnings: u64,
        spent: u64,
        clock: &Clock,
    ) -> Result<()> {
        require!(stats.has_pass, TidesError::NoSeasonPass);

        stats.total_earnings = stats
            .total_earnings
            .checked_add(earnings)
            .ok_or(TidesError::MathOverflow)?;
        stats.total_spent = stats
            .total_spent
            .checked_add(spent)
            .ok_or(TidesError::MathOverflow)?;

        let net_value = stats.total_earnings as i64 - stats.total_spent as i64;
        stats.net_value = net_value;
        stats.last_update_time = clock.unix_timestamp;

        Ok(())
    }

    pub fn end_season(season: &mut Account<Season>) -> Result<()> {
        require!(season.is_active, TidesError::SeasonAlreadyEnded);
        season.is_active = false;
        Ok(())
    }
}

