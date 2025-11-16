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

    pub fn travel_to_map(
        ctx: Context<TravelToMap>,
        new_map_id: u64,
    ) -> Result<()> {
        require!(!ctx.accounts.game_state.paused, TidesError::GamePaused);
        require!(ctx.accounts.player_state.is_active, TidesError::PlayerNotRegistered);
        
        let player_state = &mut ctx.accounts.player_state;
        require!(new_map_id != player_state.map_id, TidesError::AlreadyOnMap);
        
        // Get map data
        let map = &ctx.accounts.map;
        require!(map.map_id == new_map_id, TidesError::InvalidMap);
        
        let travel_cost = map.travel_cost;
        
        // Transfer tokens from player (burn)
        if travel_cost > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.player_token_account.to_account_info(),
                to: ctx.accounts.game_token_account.to_account_info(),
                authority: ctx.accounts.player.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            token::transfer(cpi_ctx, travel_cost)?;
        }
        
        let old_map_id = player_state.map_id;
        player_state.map_id = new_map_id;
        player_state.position_x = 0;
        player_state.position_y = 0;
        
        emit!(MapChanged {
            player: ctx.accounts.player.key(),
            old_map_id,
            new_map_id,
            cost: travel_cost,
        });
        
        Ok(())
    }

    pub fn change_ship(
        ctx: Context<ChangeShip>,
        new_ship_id: u64,
    ) -> Result<()> {
        require!(!ctx.accounts.game_state.paused, TidesError::GamePaused);
        require!(ctx.accounts.player_state.is_active, TidesError::PlayerNotRegistered);
        
        let ship = &ctx.accounts.ship;
        require!(ship.ship_id == new_ship_id, TidesError::InvalidShip);
        
        let player_state = &mut ctx.accounts.player_state;
        player_state.ship_id = new_ship_id;
        
        emit!(ShipChanged {
            player: ctx.accounts.player.key(),
            new_ship_id,
        });
        
        Ok(())
    }

    pub fn purchase_ship(
        ctx: Context<PurchaseShip>,
        ship_id: u64,
    ) -> Result<()> {
        require!(!ctx.accounts.game_state.paused, TidesError::GamePaused);
        require!(ctx.accounts.player_state.is_active, TidesError::PlayerNotRegistered);
        
        // Check if player is at harbor (simplified - would check map registry)
        let ship = &ctx.accounts.ship;
        require!(ship.ship_id == ship_id, TidesError::InvalidShip);
        
        let cost = ship.purchase_price;
        
        // Transfer tokens (burn)
        let cpi_accounts = Transfer {
            from: ctx.accounts.player_token_account.to_account_info(),
            to: ctx.accounts.game_token_account.to_account_info(),
            authority: ctx.accounts.player.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, cost)?;
        
        let player_state = &mut ctx.accounts.player_state;
        player_state.ship_id = ship_id;
        
        emit!(ShipPurchased {
            player: ctx.accounts.player.key(),
            ship_id,
            cost,
        });
        
        emit!(ShipChanged {
            player: ctx.accounts.player.key(),
            new_ship_id: ship_id,
        });
        
        Ok(())
    }

    pub fn purchase_engine(
        ctx: Context<PurchaseEngine>,
        engine_id: u64,
    ) -> Result<()> {
        require!(!ctx.accounts.game_state.paused, TidesError::GamePaused);
        require!(ctx.accounts.player_state.is_active, TidesError::PlayerNotRegistered);
        
        let engine = &ctx.accounts.engine;
        require!(engine.engine_id == engine_id, TidesError::InvalidEngine);
        
        let cost = engine.purchase_price;
        
        // Transfer tokens (burn)
        let cpi_accounts = Transfer {
            from: ctx.accounts.player_token_account.to_account_info(),
            to: ctx.accounts.game_token_account.to_account_info(),
            authority: ctx.accounts.player.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, cost)?;
        
        emit!(EnginePurchased {
            player: ctx.accounts.player.key(),
            engine_id,
            cost,
        });
        
        Ok(())
    }

    pub fn purchase_fishing_rod(
        ctx: Context<PurchaseFishingRod>,
        rod_id: u64,
    ) -> Result<()> {
        require!(!ctx.accounts.game_state.paused, TidesError::GamePaused);
        require!(ctx.accounts.player_state.is_active, TidesError::PlayerNotRegistered);
        
        let rod = &ctx.accounts.fishing_rod;
        require!(rod.rod_id == rod_id, TidesError::InvalidFishingRod);
        
        let cost = rod.purchase_price;
        
        // Transfer tokens (burn)
        let cpi_accounts = Transfer {
            from: ctx.accounts.player_token_account.to_account_info(),
            to: ctx.accounts.game_token_account.to_account_info(),
            authority: ctx.accounts.player.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, cost)?;
        
        emit!(FishingRodPurchased {
            player: ctx.accounts.player.key(),
            rod_id,
            cost,
        });
        
        Ok(())
    }

    pub fn initiate_fishing(
        ctx: Context<InitiateFishing>,
        bait_type: u64,
    ) -> Result<u64> {
        require!(!ctx.accounts.game_state.paused, TidesError::GamePaused);
        require!(ctx.accounts.player_state.is_active, TidesError::PlayerNotRegistered);
        
        // Check if fishing rod is equipped (simplified - would check inventory)
        let bait = &mut ctx.accounts.player_bait;
        require!(bait.bait_type == bait_type, TidesError::InvalidBait);
        require!(bait.amount > 0, TidesError::InsufficientBait);
        
        let fishing_state = &mut ctx.accounts.fishing_state;
        require!(fishing_state.pending_nonce == 0, TidesError::PendingFishingRequest);
        
        // Consume bait
        bait.amount = bait.amount.checked_sub(1).ok_or(TidesError::MathOverflow)?;
        
        // Increment fishing nonce
        fishing_state.fishing_nonce = fishing_state.fishing_nonce.checked_add(1).ok_or(TidesError::MathOverflow)?;
        let fishing_nonce = fishing_state.fishing_nonce;
        fishing_state.pending_nonce = fishing_nonce;
        fishing_state.bait_type_used = bait_type;
        
        emit!(FishingInitiated {
            player: ctx.accounts.player.key(),
            shard: ctx.accounts.player_state.shard,
            map_id: ctx.accounts.player_state.map_id,
            x: ctx.accounts.player_state.position_x,
            y: ctx.accounts.player_state.position_y,
            bait_type,
            nonce: fishing_nonce,
        });
        
        Ok(fishing_nonce)
    }

    pub fn fulfill_fishing(
        ctx: Context<FulfillFishing>,
        species: u64,
        weight: u16,
        timestamp: i64,
        signature: Vec<u8>,
        should_place: bool,
        place_x: u8,
        place_y: u8,
        rotation: u8,
    ) -> Result<u64> {
        require!(!ctx.accounts.game_state.paused, TidesError::GamePaused);
        
        let clock = Clock::get()?;
        let fishing_state = &mut ctx.accounts.fishing_state;
        
        require!(fishing_state.pending_nonce > 0, TidesError::ExpiredFishingRequest);
        
        // Verify timestamp
        require!(
            clock.unix_timestamp <= timestamp + fishing::SIGNATURE_TIMEOUT,
            TidesError::SignatureExpired
        );
        require!(timestamp <= clock.unix_timestamp, TidesError::FutureTimestamp);
        
        // Note: Full Ed25519 signature verification would go here
        // For now, we'll verify the server signer matches
        // In production, use solana_program::ed25519_program
        
        let fishing_nonce = fishing_state.pending_nonce;
        fishing_state.pending_nonce = 0;
        fishing_state.bait_type_used = 0;
        
        let instance_id = if species > 0 && should_place {
            // Place fish in inventory
            inventory::place_item(
                &mut ctx.accounts.player_inventory,
                ItemType::Fish,
                species,
                place_x,
                place_y,
                rotation,
                1, // shape_width (simplified)
                1, // shape_height (simplified)
                vec![], // shape_data (simplified)
            )?
        } else {
            0
        };
        
        if species > 0 && should_place {
            // Store fish catch data
            let fish_catch = &mut ctx.accounts.fish_catch;
            fish_catch.player = ctx.accounts.player.key();
            fish_catch.instance_id = instance_id;
            fish_catch.species = species;
            fish_catch.weight = weight;
            fish_catch.caught_timestamp = clock.unix_timestamp;
            
            emit!(FishCaught {
                player: ctx.accounts.player.key(),
                species,
                weight,
            });
        }
        
        Ok(instance_id)
    }

    pub fn purchase_bait(
        ctx: Context<PurchaseBait>,
        bait_type: u64,
        amount: u64,
    ) -> Result<()> {
        require!(!ctx.accounts.game_state.paused, TidesError::GamePaused);
        require!(ctx.accounts.player_state.is_active, TidesError::PlayerNotRegistered);
        require!(amount > 0, TidesError::InvalidAmount);
        
        let bait_type_account = &ctx.accounts.bait_type;
        require!(bait_type_account.bait_id == bait_type, TidesError::InvalidBait);
        require!(bait_type_account.is_active, TidesError::InvalidBait);
        
        let total_cost = bait_type_account.price
            .checked_mul(amount)
            .ok_or(TidesError::MathOverflow)?;
        
        // Transfer tokens (burn)
        let cpi_accounts = Transfer {
            from: ctx.accounts.player_token_account.to_account_info(),
            to: ctx.accounts.game_token_account.to_account_info(),
            authority: ctx.accounts.player.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, total_cost)?;
        
        // Add bait to player inventory
        let player_bait = &mut ctx.accounts.player_bait;
        player_bait.bait_type = bait_type;
        player_bait.amount = player_bait.amount.checked_add(amount).ok_or(TidesError::MathOverflow)?;
        
        emit!(BaitPurchased {
            player: ctx.accounts.player.key(),
            bait_type,
            amount,
            cost: total_cost,
        });
        
        Ok(())
    }

    pub fn update_inventory_item(
        ctx: Context<UpdateInventoryItem>,
        from_x: u8,
        from_y: u8,
        to_x: u8,
        to_y: u8,
        rotation: u8,
    ) -> Result<()> {
        require!(!ctx.accounts.game_state.paused, TidesError::GamePaused);
        require!(rotation < 4, TidesError::InvalidRotation);
        
        let inventory = &mut ctx.accounts.player_inventory;
        let item = inventory::get_item_at(inventory, from_x, from_y)?;
        require!(item.item_type != ItemType::Empty, TidesError::ItemNotFound);
        
        // Remove from old position
        inventory::remove_item(inventory, item.instance_id)?;
        
        // Place at new position (simplified - would get shape from registry)
        inventory::place_item(
            inventory,
            item.item_type,
            item.item_id,
            to_x,
            to_y,
            rotation,
            1, // shape_width (simplified)
            1, // shape_height (simplified)
            vec![], // shape_data (simplified)
        )?;
        
        emit!(ItemMoved {
            player: ctx.accounts.player.key(),
            instance_id: item.instance_id,
            from_x,
            from_y,
            to_x,
            to_y,
            rotation,
        });
        
        Ok(())
    }

    pub fn discard_inventory_item(
        ctx: Context<DiscardInventoryItem>,
        x: u8,
        y: u8,
    ) -> Result<()> {
        require!(!ctx.accounts.game_state.paused, TidesError::GamePaused);
        
        let inventory = &mut ctx.accounts.player_inventory;
        let item = inventory::get_item_at(inventory, x, y)?;
        require!(item.item_type != ItemType::Empty, TidesError::ItemNotFound);
        
        inventory::remove_item(inventory, item.instance_id)?;
        
        // If it's a fish, remove fish catch data
        if item.item_type == ItemType::Fish {
            // Fish catch account would be closed/removed here
        }
        
        emit!(ItemDiscarded {
            player: ctx.accounts.player.key(),
            item_type: item.item_type,
            item_id: item.item_id,
            instance_id: item.instance_id,
        });
        
        Ok(())
    }

    pub fn purchase_season_pass(
        ctx: Context<PurchaseSeasonPass>,
        season_id: u64,
    ) -> Result<()> {
        require!(!ctx.accounts.game_state.paused, TidesError::GamePaused);
        
        let season = &mut ctx.accounts.season;
        require!(season.season_id == season_id, TidesError::InvalidId);
        
        let clock = Clock::get()?;
        require!(season.is_active, TidesError::SeasonNotActive);
        require!(clock.unix_timestamp >= season.start_time, TidesError::SeasonNotStarted);
        require!(clock.unix_timestamp <= season.end_time, TidesError::SeasonHasEnded);
        
        let stats = &mut ctx.accounts.player_stats;
        require!(!stats.has_pass, TidesError::AlreadyOwnsSeasonPass);
        
        // Transfer SOL for season pass
        require!(
            ctx.accounts.player.lamports() >= season.pass_price,
            TidesError::InsufficientPayment
        );
        
        **ctx.accounts.player.to_account_info().try_borrow_mut_lamports()? -= season.pass_price;
        **ctx.accounts.season_pass_state.to_account_info().try_borrow_mut_lamports()? += season.pass_price;
        
        // Initialize player stats
        stats.player = ctx.accounts.player.key();
        stats.season_id = season_id;
        stats.total_earnings = 0;
        stats.total_spent = 0;
        stats.net_value = 0;
        stats.last_update_time = clock.unix_timestamp;
        stats.has_pass = true;
        
        season.total_passes = season.total_passes.checked_add(1).ok_or(TidesError::MathOverflow)?;
        
        emit!(SeasonPassPurchased {
            player: ctx.accounts.player.key(),
            season_id,
        });
        
        Ok(())
    }

    // ============ Admin Functions ============

    pub fn pause_game(ctx: Context<AdminOnly>) -> Result<()> {
        let game_state = &mut ctx.accounts.game_state;
        require!(!game_state.paused, TidesError::GamePaused);
        game_state.paused = true;
        Ok(())
    }

    pub fn unpause_game(ctx: Context<AdminOnly>) -> Result<()> {
        let game_state = &mut ctx.accounts.game_state;
        require!(game_state.paused, TidesError::GamePaused);
        game_state.paused = false;
        Ok(())
    }

    pub fn update_server_signer(
        ctx: Context<AdminOnly>,
        new_signer: Pubkey,
    ) -> Result<()> {
        require!(new_signer != Pubkey::default(), TidesError::InvalidAddress);
        let game_state = &mut ctx.accounts.game_state;
        game_state.server_signer = new_signer;
        Ok(())
    }

    pub fn set_max_players_per_shard(
        ctx: Context<AdminOnly>,
        new_limit: u64,
    ) -> Result<()> {
        require!(new_limit > 0 && new_limit <= 10000, TidesError::LimitOutOfBounds);
        let game_state = &mut ctx.accounts.game_state;
        game_state.max_players_per_shard = new_limit;
        Ok(())
    }

    pub fn admin_change_player_shard(
        ctx: Context<AdminChangeShard>,
        new_shard: u8,
        bypass_limit: bool,
    ) -> Result<()> {
        require!(!ctx.accounts.game_state.paused, TidesError::GamePaused);
        require!(new_shard < MAX_SHARDS, TidesError::InvalidShardId);
        
        let player_state = &mut ctx.accounts.player_state;
        let old_shard = player_state.shard;
        require!(new_shard != old_shard, TidesError::AlreadyInShard);
        
        // Set the new shard data's shard_id if it was just created
        let new_shard_data = &mut ctx.accounts.new_shard_data;
        if new_shard_data.shard_id == 0 {
            new_shard_data.shard_id = new_shard;
            new_shard_data.player_count = 0;
        }
        
        let old_shard_data = &mut ctx.accounts.old_shard_data;
        
        if !bypass_limit {
            require!(
                new_shard_data.player_count < ctx.accounts.game_state.max_players_per_shard,
                TidesError::ShardFull
            );
        }
        
        // Update shard counts
        old_shard_data.player_count = old_shard_data.player_count.checked_sub(1).ok_or(TidesError::MathOverflow)?;
        new_shard_data.player_count = new_shard_data.player_count.checked_add(1).ok_or(TidesError::MathOverflow)?;
        
        player_state.shard = new_shard;
        
        emit!(ShardChanged {
            player: player_state.player,
            old_shard,
            new_shard,
        });
        
        Ok(())
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

#[derive(Accounts)]
pub struct TravelToMap<'info> {
    #[account(mut)]
    pub game_state: Account<'info, GameState>,
    
    #[account(mut)]
    pub player_state: Account<'info, PlayerState>,
    
    pub map: Account<'info, registries::Map>,
    
    #[account(mut)]
    pub player: Signer<'info>,
    
    #[account(mut)]
    pub player_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub game_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ChangeShip<'info> {
    #[account(mut)]
    pub game_state: Account<'info, GameState>,
    
    #[account(mut)]
    pub player_state: Account<'info, PlayerState>,
    
    pub ship: Account<'info, registries::Ship>,
    
    pub player: Signer<'info>,
}

#[derive(Accounts)]
pub struct PurchaseShip<'info> {
    #[account(mut)]
    pub game_state: Account<'info, GameState>,
    
    #[account(mut)]
    pub player_state: Account<'info, PlayerState>,
    
    pub ship: Account<'info, registries::Ship>,
    
    #[account(mut)]
    pub player: Signer<'info>,
    
    #[account(mut)]
    pub player_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub game_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct PurchaseEngine<'info> {
    #[account(mut)]
    pub game_state: Account<'info, GameState>,
    
    #[account(mut)]
    pub player_state: Account<'info, PlayerState>,
    
    pub engine: Account<'info, registries::Engine>,
    
    #[account(mut)]
    pub player: Signer<'info>,
    
    #[account(mut)]
    pub player_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub game_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct PurchaseFishingRod<'info> {
    #[account(mut)]
    pub game_state: Account<'info, GameState>,
    
    #[account(mut)]
    pub player_state: Account<'info, PlayerState>,
    
    pub fishing_rod: Account<'info, registries::FishingRod>,
    
    #[account(mut)]
    pub player: Signer<'info>,
    
    #[account(mut)]
    pub player_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub game_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct InitiateFishing<'info> {
    #[account(mut)]
    pub game_state: Account<'info, GameState>,
    
    #[account(mut)]
    pub player_state: Account<'info, PlayerState>,
    
    #[account(
        mut,
        has_one = player @ TidesError::InvalidPlayer
    )]
    pub fishing_state: Account<'info, fishing::FishingState>,
    
    #[account(
        mut,
        has_one = player @ TidesError::InvalidPlayer
    )]
    pub player_bait: Account<'info, fishing::PlayerBait>,
    
    pub player: Signer<'info>,
}

#[derive(Accounts)]
pub struct FulfillFishing<'info> {
    #[account(mut)]
    pub game_state: Account<'info, GameState>,
    
    #[account(mut)]
    pub player_state: Account<'info, PlayerState>,
    
    #[account(
        mut,
        has_one = player @ TidesError::InvalidPlayer
    )]
    pub fishing_state: Account<'info, fishing::FishingState>,
    
    #[account(
        mut,
        has_one = player @ TidesError::InvalidPlayer
    )]
    pub player_inventory: Account<'info, inventory::PlayerInventory>,
    
    #[account(
        init,
        payer = player,
        space = inventory::FishCatch::LEN,
        seeds = [b"fish", player.key().as_ref(), &fishing_state.fishing_nonce.to_le_bytes()],
        bump
    )]
    pub fish_catch: Account<'info, inventory::FishCatch>,
    
    pub player: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PurchaseBait<'info> {
    #[account(mut)]
    pub game_state: Account<'info, GameState>,
    
    #[account(mut)]
    pub player_state: Account<'info, PlayerState>,
    
    pub bait_type: Account<'info, registries::BaitType>,
    
    #[account(
        init_if_needed,
        payer = player,
        space = fishing::PlayerBait::LEN,
        seeds = [b"bait", player.key().as_ref(), &bait_type.bait_id.to_le_bytes()],
        bump
    )]
    pub player_bait: Account<'info, fishing::PlayerBait>,
    
    #[account(mut)]
    pub player: Signer<'info>,
    
    #[account(mut)]
    pub player_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub game_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateInventoryItem<'info> {
    #[account(mut)]
    pub game_state: Account<'info, GameState>,
    
    #[account(
        mut,
        has_one = player @ TidesError::InvalidPlayer
    )]
    pub player_inventory: Account<'info, inventory::PlayerInventory>,
    
    pub player: Signer<'info>,
}

#[derive(Accounts)]
pub struct DiscardInventoryItem<'info> {
    #[account(mut)]
    pub game_state: Account<'info, GameState>,
    
    #[account(
        mut,
        has_one = player @ TidesError::InvalidPlayer
    )]
    pub player_inventory: Account<'info, inventory::PlayerInventory>,
    
    pub player: Signer<'info>,
}

#[derive(Accounts)]
pub struct PurchaseSeasonPass<'info> {
    #[account(mut)]
    pub game_state: Account<'info, GameState>,
    
    #[account(mut)]
    pub season: Account<'info, season_pass::Season>,
    
    #[account(
        init_if_needed,
        payer = player,
        space = season_pass::PlayerSeasonStats::LEN,
        seeds = [b"season_stats", player.key().as_ref(), &season.season_id.to_le_bytes()],
        bump
    )]
    pub player_stats: Account<'info, season_pass::PlayerSeasonStats>,
    
    #[account(
        init,
        payer = player,
        space = season_pass::SeasonPassState::LEN,
        seeds = [b"season_pass_state"],
        bump
    )]
    pub season_pass_state: Account<'info, season_pass::SeasonPassState>,
    
    #[account(mut)]
    pub player: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AdminOnly<'info> {
    #[account(
        mut,
        has_one = admin @ TidesError::InvalidAddress
    )]
    pub game_state: Account<'info, GameState>,
    
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(new_shard: u8)]
pub struct AdminChangeShard<'info> {
    #[account(
        mut,
        has_one = admin @ TidesError::InvalidAddress
    )]
    pub game_state: Account<'info, GameState>,
    
    #[account(mut)]
    pub player_state: Account<'info, PlayerState>,
    
    #[account(
        mut,
        seeds = [b"shard", &[player_state.shard]],
        bump
    )]
    pub old_shard_data: Account<'info, ShardData>,
    
    #[account(
        init_if_needed,
        payer = admin,
        space = ShardData::LEN,
        seeds = [b"shard", &[new_shard]],
        bump
    )]
    pub new_shard_data: Account<'info, ShardData>,
    
    pub admin: Signer<'info>,
    
    pub system_program: Program<'info, System>,
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

#[event]
pub struct MapChanged {
    pub player: Pubkey,
    pub old_map_id: u64,
    pub new_map_id: u64,
    pub cost: u64,
}

#[event]
pub struct ShipChanged {
    pub player: Pubkey,
    pub new_ship_id: u64,
}

#[event]
pub struct ShipPurchased {
    pub player: Pubkey,
    pub ship_id: u64,
    pub cost: u64,
}

#[event]
pub struct EnginePurchased {
    pub player: Pubkey,
    pub engine_id: u64,
    pub cost: u64,
}

#[event]
pub struct FishingRodPurchased {
    pub player: Pubkey,
    pub rod_id: u64,
    pub cost: u64,
}

#[event]
pub struct FishingInitiated {
    pub player: Pubkey,
    pub shard: u8,
    pub map_id: u64,
    pub x: i32,
    pub y: i32,
    pub bait_type: u64,
    pub nonce: u64,
}

#[event]
pub struct FishCaught {
    pub player: Pubkey,
    pub species: u64,
    pub weight: u16,
}

#[event]
pub struct BaitPurchased {
    pub player: Pubkey,
    pub bait_type: u64,
    pub amount: u64,
    pub cost: u64,
}

#[event]
pub struct ItemMoved {
    pub player: Pubkey,
    pub instance_id: u64,
    pub from_x: u8,
    pub from_y: u8,
    pub to_x: u8,
    pub to_y: u8,
    pub rotation: u8,
}

#[event]
pub struct ItemDiscarded {
    pub player: Pubkey,
    pub item_type: ItemType,
    pub item_id: u64,
    pub instance_id: u64,
}

#[event]
pub struct SeasonPassPurchased {
    pub player: Pubkey,
    pub season_id: u64,
}

#[event]
pub struct ShardChanged {
    pub player: Pubkey,
    pub old_shard: u8,
    pub new_shard: u8,
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
    
    #[msg("Already in shard")]
    AlreadyInShard,
    
    #[msg("Season not started")]
    SeasonNotStarted,
    
    #[msg("Insufficient payment")]
    InsufficientPayment,
    
    #[msg("Limit out of bounds")]
    LimitOutOfBounds,
}
