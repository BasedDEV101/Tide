use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

// Import modules
pub mod fishing;
pub mod inventory;
pub mod registries;
pub mod season_pass;

// Re-export for convenience
pub use fishing::*;
pub use inventory::*;
pub use registries::*;
pub use season_pass::*;

declare_id!("Tides1111111111111111111111111111111111111111");

// ============ Constants ============
pub const FUEL_PRICE_PER_UNIT: u64 = 10_000_000_000; // 10 TTC (tides token) per fuel unit (with 9 decimals)
pub const MAX_SHARDS: u8 = 100;
pub const HEX_MOVE_COST: u64 = 1_000_000_000; // Base fuel cost per hex (1 TTC)
pub const BASE_MOVEMENT_SPEED: u64 = 1000; // Base movement speed (lower = faster)
pub const PRICE_DECAY_RATE: u64 = 5; // 5% decrease per fish sale
pub const PRICE_RECOVERY_RATE: u64 = 463; // ~100% in 6 hours
pub const FRESHNESS_DECAY_PERIOD: i64 = 900; // 15 minutes in seconds
pub const FRESHNESS_DECAY_RATE: u64 = 25; // 25%

// Movement constraints
pub const MAX_COORDINATE: i32 = 1000;
pub const MIN_COORDINATE: i32 = -1000;

// Hex movement directions (0=NE, 1=E, 2=SE, 3=SW, 4=W, 5=NW)
pub const HEX_DIRECTIONS_X: [i32; 6] = [1, 1, 0, -1, -1, 0];
pub const HEX_DIRECTIONS_Y: [i32; 6] = [0, -1, -1, 0, 1, 1];

// ============ Enums ============

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum SlotType {
    Normal,      // 0 - Regular cargo slot
    Engine,      // 1 - Engine slot
    FishingRod,  // 2 - Fishing rod slot
    Blocked,     // 3 - Blocked slot
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum ItemType {
    Empty,       // 0 - No item
    Fish,        // 1 - Fish items
    Engine,      // 2 - Engine items
    FishingRod,  // 3 - Fishing rod items
}

// ============ Account Structures ============

#[account]
pub struct GameState {
    pub currency_mint: Pubkey,
    pub admin: Pubkey,
    pub paused: bool,
    pub max_players_per_shard: u64,
    pub server_signer: Pubkey,
}

#[account]
pub struct PlayerState {
    pub player: Pubkey,
    pub map_id: u64,
    pub ship_id: u64,
    pub current_fuel: u64,
    pub last_move_timestamp: i64,
    pub next_move_time: i64,
    pub movement_speed: u64,
    pub position_x: i32,
    pub position_y: i32,
    pub shard: u8,
    pub is_active: bool,
}

impl PlayerState {
    pub const LEN: usize = 8 + // discriminator
        32 + // player
        8 + // map_id
        8 + // ship_id
        8 + // current_fuel
        8 + // last_move_timestamp
        8 + // next_move_time
        8 + // movement_speed
        4 + // position_x
        4 + // position_y
        1 + // shard
        1; // is_active
}

#[account]
pub struct FishMarketData {
    pub species: u64,
    pub value: u64,
    pub last_sold_timestamp: i64,
}

impl FishMarketData {
    pub const LEN: usize = 8 + // discriminator
        8 + // species
        8 + // value
        8; // last_sold_timestamp
}

#[account]
pub struct ShardData {
    pub shard_id: u8,
    pub player_count: u64,
}

impl ShardData {
    pub const LEN: usize = 8 + // discriminator
        1 + // shard_id
        8; // player_count
}

// ============ Program Module ============

#[program]
pub mod tides {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        currency_mint: Pubkey,
        server_signer: Pubkey,
        max_players_per_shard: u64,
    ) -> Result<()> {
        let game_state = &mut ctx.accounts.game_state;
        game_state.currency_mint = currency_mint;
        game_state.admin = ctx.accounts.admin.key();
        game_state.paused = false;
        game_state.max_players_per_shard = max_players_per_shard;
        game_state.server_signer = server_signer;
        Ok(())
    }

    pub fn register_player(
        ctx: Context<RegisterPlayer>,
        shard: u8,
        map_id: u64,
    ) -> Result<()> {
        require!(!ctx.accounts.game_state.paused, TidesError::GamePaused);
        require!(shard < MAX_SHARDS, TidesError::InvalidShardId);
        require!(!ctx.accounts.player_state.is_active, TidesError::PlayerAlreadyRegistered);

        // Initialize player state
        let player_state = &mut ctx.accounts.player_state;
        player_state.player = ctx.accounts.player.key();
        player_state.map_id = map_id;
        player_state.ship_id = 1; // Default ship
        player_state.current_fuel = 100_000_000_000; // 100 fuel units
        player_state.last_move_timestamp = Clock::get()?.unix_timestamp;
        player_state.next_move_time = Clock::get()?.unix_timestamp;
        player_state.movement_speed = BASE_MOVEMENT_SPEED;
        player_state.position_x = 0;
        player_state.position_y = 0;
        player_state.shard = shard;
        player_state.is_active = true;

        // Update shard count
        let shard_data = &mut ctx.accounts.shard_data;
        shard_data.shard_id = shard;
        shard_data.player_count = shard_data.player_count.checked_add(1).ok_or(TidesError::MathOverflow)?;

        require!(
            shard_data.player_count <= ctx.accounts.game_state.max_players_per_shard,
            TidesError::ShardFull
        );

        emit!(PlayerRegistered {
            player: ctx.accounts.player.key(),
            shard,
        });

        Ok(())
    }

    pub fn move_player(
        ctx: Context<MovePlayer>,
        directions: Vec<u8>,
    ) -> Result<()> {
        require!(!ctx.accounts.game_state.paused, TidesError::GamePaused);
        require!(ctx.accounts.player_state.is_active, TidesError::PlayerNotRegistered);
        
        let clock = Clock::get()?;
        let player_state = &mut ctx.accounts.player_state;
        
        require!(
            clock.unix_timestamp >= player_state.next_move_time,
            TidesError::OnCooldown
        );
        
        require!(!directions.is_empty(), TidesError::NoDirectionsProvided);
        require!(directions.len() <= 20, TidesError::TooManyMoves);

        // Calculate fuel cost
        let fuel_cost = calculate_fuel_cost(directions.len() as u64);
        require!(
            player_state.current_fuel >= fuel_cost,
            TidesError::InsufficientFuel
        );

        // Move player (simplified - would need map validation in full implementation)
        let mut new_x = player_state.position_x;
        let mut new_y = player_state.position_y;

        for direction in &directions {
            require!(*direction < 6, TidesError::InvalidDirection);
            new_x = new_x
                .checked_add(HEX_DIRECTIONS_X[*direction as usize])
                .ok_or(TidesError::MathOverflow)?;
            new_y = new_y
                .checked_add(HEX_DIRECTIONS_Y[*direction as usize])
                .ok_or(TidesError::MathOverflow)?;
        }

        // Update position and fuel
        player_state.position_x = new_x;
        player_state.position_y = new_y;
        player_state.current_fuel = player_state
            .current_fuel
            .checked_sub(fuel_cost)
            .ok_or(TidesError::MathOverflow)?;
        player_state.last_move_timestamp = clock.unix_timestamp;
        player_state.next_move_time = clock.unix_timestamp
            .checked_add((player_state.movement_speed * directions.len() as u64) as i64)
            .ok_or(TidesError::MathOverflow)?;

        emit!(PlayerMoved {
            player: ctx.accounts.player.key(),
            shard: player_state.shard,
            map_id: player_state.map_id,
            x: new_x,
            y: new_y,
            fuel_consumed: fuel_cost,
        });

        Ok(())
    }

    pub fn purchase_fuel(
        ctx: Context<PurchaseFuel>,
        amount: u64,
    ) -> Result<()> {
        require!(!ctx.accounts.game_state.paused, TidesError::GamePaused);
        require!(ctx.accounts.player_state.is_active, TidesError::PlayerNotRegistered);
        require!(amount > 0, TidesError::InvalidAmount);

        let total_cost = amount
            .checked_mul(FUEL_PRICE_PER_UNIT)
            .ok_or(TidesError::MathOverflow)?;

        // Transfer tokens from player to game (burn)
        let cpi_accounts = Transfer {
            from: ctx.accounts.player_token_account.to_account_info(),
            to: ctx.accounts.game_token_account.to_account_info(),
            authority: ctx.accounts.player.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, total_cost)?;

        // Add fuel to player
        let player_state = &mut ctx.accounts.player_state;
        player_state.current_fuel = player_state
            .current_fuel
            .checked_add(amount.checked_mul(1_000_000_000).ok_or(TidesError::MathOverflow)?)
            .ok_or(TidesError::MathOverflow)?;

        emit!(FuelPurchased {
            player: ctx.accounts.player.key(),
            amount,
            cost: total_cost,
        });

        Ok(())
    }

    pub fn sell_fish(
        ctx: Context<SellFish>,
        instance_id: u64,
        species: u64,
        weight: u16,
        caught_timestamp: i64,
    ) -> Result<u64> {
        require!(!ctx.accounts.game_state.paused, TidesError::GamePaused);
        require!(ctx.accounts.player_state.is_active, TidesError::PlayerNotRegistered);
        require!(species > 0, TidesError::InvalidSpecies);

        let clock = Clock::get()?;
        let freshness = calculate_fish_freshness(caught_timestamp, clock.unix_timestamp);
        
        let sale_price = update_fish_market_data(
            &mut ctx.accounts.fish_market,
            species,
            weight,
            freshness,
            clock.unix_timestamp,
        )?;

        // Mint tokens to player
        // In full implementation, would call SPL token mint instruction
        // This is simplified for structure

        emit!(FishSold {
            species,
            weight: weight as u64,
            freshness,
            sale_price,
        });

        Ok(sale_price)
    }

    // ============ Helper Functions ============

    fn calculate_fuel_cost(distance: u64) -> u64 {
        distance
            .checked_mul(HEX_MOVE_COST)
            .unwrap_or(u64::MAX)
    }

    fn calculate_fish_freshness(caught_at: i64, current_time: i64) -> u64 {
        let seconds_elapsed = current_time
            .checked_sub(caught_at)
            .unwrap_or(0);
        
        if seconds_elapsed < 0 {
            return 100;
        }

        let decay_periods = seconds_elapsed
            .checked_div(FRESHNESS_DECAY_PERIOD)
            .unwrap_or(0);
        
        let freshness_decayed = (decay_periods as u64)
            .checked_mul(FRESHNESS_DECAY_RATE)
            .unwrap_or(100);

        if freshness_decayed >= 100 {
            return 0;
        }

        100 - freshness_decayed
    }

    fn update_fish_market_data(
        market_data: &mut Account<FishMarketData>,
        species: u64,
        weight: u16,
        freshness: u64,
        current_timestamp: i64,
    ) -> Result<u64> {
        let base_price = 100_000_000; // Simplified - would come from fish registry
        
        let sale_price = if market_data.last_sold_timestamp == 0 {
            // First sale
            base_price
                .checked_mul(weight as u64)
                .and_then(|p| p.checked_mul(freshness))
                .and_then(|p| p.checked_div(100))
                .ok_or(TidesError::MathOverflow)?;
            
            market_data.value = base_price
                .checked_sub(base_price.checked_mul(PRICE_DECAY_RATE).and_then(|p| p.checked_div(100)).unwrap_or(0))
                .unwrap_or(0);
            
            base_price
                .checked_mul(weight as u64)
                .and_then(|p| p.checked_mul(freshness))
                .and_then(|p| p.checked_div(100))
                .ok_or(TidesError::MathOverflow)?
        } else {
            // Calculate market recovery
            let seconds_elapsed = current_timestamp
                .checked_sub(market_data.last_sold_timestamp)
                .unwrap_or(0);
            
            let market_value = market_data.value
                .checked_add(base_price.checked_mul(PRICE_RECOVERY_RATE).and_then(|p| p.checked_mul(seconds_elapsed as u64)).and_then(|p| p.checked_div(10_000_000)).unwrap_or(0))
                .unwrap_or(market_data.value);
            
            let capped_value = market_value.min(base_price);
            
            let sale_price = capped_value
                .checked_mul(weight as u64)
                .and_then(|p| p.checked_mul(freshness))
                .and_then(|p| p.checked_div(100))
                .ok_or(TidesError::MathOverflow)?;
            
            market_data.value = capped_value
                .checked_sub(capped_value.checked_mul(PRICE_DECAY_RATE).and_then(|p| p.checked_div(100)).unwrap_or(0))
                .unwrap_or(0);
            
            sale_price
        };

        market_data.last_sold_timestamp = current_timestamp;
        market_data.species = species;

        Ok(sale_price)
    }
}

// ============ Contexts ============

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + 32 + 32 + 1 + 8 + 32
    )]
    pub game_state: Account<'info, GameState>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterPlayer<'info> {
    #[account(mut)]
    pub game_state: Account<'info, GameState>,
    
    #[account(
        init,
        payer = player,
        space = PlayerState::LEN
    )]
    pub player_state: Account<'info, PlayerState>,
    
    #[account(
        init,
        payer = player,
        space = ShardData::LEN,
        seeds = [b"shard", &[shard]],
        bump
    )]
    pub shard_data: Account<'info, ShardData>,
    
    #[account(mut)]
    pub player: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MovePlayer<'info> {
    #[account(mut)]
    pub game_state: Account<'info, GameState>,
    
    #[account(
        mut,
        has_one = player @ TidesError::InvalidPlayer
    )]
    pub player_state: Account<'info, PlayerState>,
    
    pub player: Signer<'info>,
}

#[derive(Accounts)]
pub struct PurchaseFuel<'info> {
    #[account(mut)]
    pub game_state: Account<'info, GameState>,
    
    #[account(mut)]
    pub player_state: Account<'info, PlayerState>,
    
    #[account(mut)]
    pub player: Signer<'info>,
    
    #[account(mut)]
    pub player_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub game_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct SellFish<'info> {
    #[account(mut)]
    pub game_state: Account<'info, GameState>,
    
    #[account(mut)]
    pub player_state: Account<'info, PlayerState>,
    
    #[account(mut)]
    pub fish_market: Account<'info, FishMarketData>,
    
    pub player: Signer<'info>,
}

// ============ Events ============

#[event]
pub struct PlayerRegistered {
    pub player: Pubkey,
    pub shard: u8,
}

#[event]
pub struct PlayerMoved {
    pub player: Pubkey,
    pub shard: u8,
    pub map_id: u64,
    pub x: i32,
    pub y: i32,
    pub fuel_consumed: u64,
}

#[event]
pub struct FuelPurchased {
    pub player: Pubkey,
    pub amount: u64,
    pub cost: u64,
}

#[event]
pub struct FishSold {
    pub species: u64,
    pub weight: u64,
    pub freshness: u64,
    pub sale_price: u64,
}

// ============ Errors ============

#[error_code]
pub enum TidesError {
    #[msg("Game is paused")]
    GamePaused,
    
    #[msg("Player not registered")]
    PlayerNotRegistered,
    
    #[msg("Player already registered")]
    PlayerAlreadyRegistered,
    
    #[msg("Invalid shard ID")]
    InvalidShardId,
    
    #[msg("Shard is full")]
    ShardFull,
    
    #[msg("Invalid direction")]
    InvalidDirection,
    
    #[msg("No directions provided")]
    NoDirectionsProvided,
    
    #[msg("Too many moves")]
    TooManyMoves,
    
    #[msg("Insufficient fuel")]
    InsufficientFuel,
    
    #[msg("On cooldown")]
    OnCooldown,
    
    #[msg("Invalid amount")]
    InvalidAmount,
    
    #[msg("Invalid species")]
    InvalidSpecies,
    
    #[msg("Math overflow")]
    MathOverflow,
    
    #[msg("Invalid player")]
    InvalidPlayer,
    
    #[msg("Invalid address")]
    InvalidAddress,
    
    #[msg("Insufficient balance")]
    InsufficientBalance,
    
    #[msg("Invalid dimensions")]
    InvalidDimensions,
    
    #[msg("Invalid rotation")]
    InvalidRotation,
    
    #[msg("Array length mismatch")]
    ArrayLengthMismatch,
    
    #[msg("Position out of bounds")]
    PositionOutOfBounds,
    
    #[msg("Position occupied")]
    PositionOccupied,
    
    #[msg("Item not found")]
    ItemNotFound,
    
    #[msg("Cannot place item")]
    CannotPlaceItem,
    
    #[msg("Invalid item type")]
    InvalidItemType,
    
    #[msg("No fishing rod equipped")]
    NoFishingRodEquipped,
    
    #[msg("Pending fishing request")]
    PendingFishingRequest,
    
    #[msg("Invalid fishing result")]
    InvalidFishingResult,
    
    #[msg("Signature expired")]
    SignatureExpired,
    
    #[msg("Future timestamp")]
    FutureTimestamp,
    
    #[msg("Expired fishing request")]
    ExpiredFishingRequest,
    
    #[msg("Invalid bait")]
    InvalidBait,
    
    #[msg("Insufficient bait")]
    InsufficientBait,
    
    #[msg("Not at harbor")]
    NotAtHarbor,
    
    #[msg("Invalid ship")]
    InvalidShip,
    
    #[msg("Invalid engine")]
    InvalidEngine,
    
    #[msg("Invalid fishing rod")]
    InvalidFishingRod,
    
    #[msg("Invalid map")]
    InvalidMap,
    
    #[msg("Already on map")]
    AlreadyOnMap,
    
    #[msg("Operation failed")]
    OperationFailed,
    
    #[msg("Empty string")]
    EmptyString,
    
    #[msg("Start time not in future")]
    StartTimeNotInFuture,
    
    #[msg("Invalid time range")]
    InvalidTimeRange,
    
    #[msg("No active season")]
    NoActiveSeason,
    
    #[msg("Season not active")]
    SeasonNotActive,
    
    #[msg("Season already ended")]
    SeasonAlreadyEnded,
    
    #[msg("No season pass")]
    NoSeasonPass,
    
    #[msg("Already owns season pass")]
    AlreadyOwnsSeasonPass,
    
    #[msg("Invalid ID")]
    InvalidId,
}
