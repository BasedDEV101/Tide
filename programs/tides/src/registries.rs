use anchor_lang::prelude::*;
use crate::{TidesError, SlotType};

// ============ Registry Account Structures ============

#[account]
pub struct FishRegistry {
    pub admin: Pubkey,
    pub species_count: u64,
}

impl FishRegistry {
    pub const LEN: usize = 8 + // discriminator
        32 + // admin
        8; // species_count
}

#[account]
pub struct FishSpecies {
    pub registry: Pubkey,
    pub species_id: u64,
    pub base_price: u64,
    pub shape_width: u8,
    pub shape_height: u8,
    pub shape_data: Vec<u8>,
}

impl FishSpecies {
    pub fn calculate_size(shape_data_len: usize) -> usize {
        8 + // discriminator
        32 + // registry
        8 + // species_id
        8 + // base_price
        1 + // shape_width
        1 + // shape_height
        4 + shape_data_len // shape_data Vec
    }
}

#[account]
pub struct BaitType {
    pub registry: Pubkey,
    pub bait_id: u64,
    pub name: String,
    pub price: u64,
    pub is_active: bool,
}

impl BaitType {
    pub fn calculate_size(name_len: usize) -> usize {
        8 + // discriminator
        32 + // registry
        8 + // bait_id
        4 + name_len + // name String
        8 + // price
        1 // is_active
    }
}

#[account]
pub struct ShipRegistry {
    pub admin: Pubkey,
    pub ship_count: u64,
}

impl ShipRegistry {
    pub const LEN: usize = 8 + // discriminator
        32 + // admin
        8; // ship_count
}

#[account]
pub struct Ship {
    pub registry: Pubkey,
    pub ship_id: u64,
    pub cargo_width: u8,
    pub cargo_height: u8,
    pub purchase_price: u64,
    pub durability: u64, // Used as proxy for ship weight
    pub slot_types: Vec<SlotType>,
}

impl Ship {
    pub fn calculate_size(cargo_width: u8, cargo_height: u8) -> usize {
        8 + // discriminator
        32 + // registry
        8 + // ship_id
        1 + // cargo_width
        1 + // cargo_height
        8 + // purchase_price
        8 + // durability
        4 + (cargo_width as usize * cargo_height as usize) // slot_types Vec
    }
}

#[account]
pub struct EngineRegistry {
    pub admin: Pubkey,
    pub engine_count: u64,
}

impl EngineRegistry {
    pub const LEN: usize = 8 + // discriminator
        32 + // admin
        8; // engine_count
}

#[account]
pub struct Engine {
    pub registry: Pubkey,
    pub engine_id: u64,
    pub engine_power: u64,
    pub fuel_consumption_rate: u64,
    pub purchase_price: u64,
    pub shape_width: u8,
    pub shape_height: u8,
    pub shape_data: Vec<u8>,
}

impl Engine {
    pub fn calculate_size(shape_data_len: usize) -> usize {
        8 + // discriminator
        32 + // registry
        8 + // engine_id
        8 + // engine_power
        8 + // fuel_consumption_rate
        8 + // purchase_price
        1 + // shape_width
        1 + // shape_height
        4 + shape_data_len // shape_data Vec
    }
}

#[account]
pub struct FishingRodRegistry {
    pub admin: Pubkey,
    pub rod_count: u64,
}

impl FishingRodRegistry {
    pub const LEN: usize = 8 + // discriminator
        32 + // admin
        8; // rod_count
}

#[account]
pub struct FishingRod {
    pub registry: Pubkey,
    pub rod_id: u64,
    pub purchase_price: u64,
    pub shape_width: u8,
    pub shape_height: u8,
    pub shape_data: Vec<u8>,
}

impl FishingRod {
    pub fn calculate_size(shape_data_len: usize) -> usize {
        8 + // discriminator
        32 + // registry
        8 + // rod_id
        8 + // purchase_price
        1 + // shape_width
        1 + // shape_height
        4 + shape_data_len // shape_data Vec
    }
}

#[account]
pub struct MapRegistry {
    pub admin: Pubkey,
    pub map_count: u64,
}

impl MapRegistry {
    pub const LEN: usize = 8 + // discriminator
        32 + // admin
        8; // map_count
}

#[account]
pub struct Map {
    pub registry: Pubkey,
    pub map_id: u64,
    pub travel_cost: u64,
    pub min_x: i32,
    pub max_x: i32,
    pub min_y: i32,
    pub max_y: i32,
    pub harbors: Vec<(i32, i32)>, // Harbor positions
}

impl Map {
    pub fn calculate_size(harbor_count: usize) -> usize {
        8 + // discriminator
        32 + // registry
        8 + // map_id
        8 + // travel_cost
        4 + // min_x
        4 + // max_x
        4 + // min_y
        4 + // max_y
        4 + (harbor_count * 8) // harbors Vec (each (i32, i32) = 8 bytes)
    }
}

// ============ Registry Module Functions ============

pub mod registries {
    use super::*;

    pub fn is_valid_species(fish_species: &Account<FishSpecies>, species_id: u64) -> bool {
        fish_species.species_id == species_id
    }

    pub fn is_valid_bait(bait_type: &Account<BaitType>, bait_id: u64) -> bool {
        bait_type.bait_id == bait_id && bait_type.is_active
    }

    pub fn is_valid_ship(ship: &Account<Ship>, ship_id: u64) -> bool {
        ship.ship_id == ship_id
    }

    pub fn is_valid_engine(engine: &Account<Engine>, engine_id: u64) -> bool {
        engine.engine_id == engine_id
    }

    pub fn is_valid_rod(rod: &Account<FishingRod>, rod_id: u64) -> bool {
        rod.rod_id == rod_id
    }

    pub fn is_valid_map(map: &Account<Map>, map_id: u64) -> bool {
        map.map_id == map_id
    }

    pub fn is_valid_position(map: &Account<Map>, x: i32, y: i32) -> bool {
        x >= map.min_x && x <= map.max_x && y >= map.min_y && y <= map.max_y
    }

    pub fn is_harbor(map: &Account<Map>, x: i32, y: i32) -> bool {
        map.harbors.iter().any(|(hx, hy)| *hx == x && *hy == y)
    }
}

