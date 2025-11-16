use anchor_lang::prelude::*;
use crate::{TidesError, SlotType, ItemType};

// ============ Inventory Account Structures ============

#[account]
pub struct PlayerInventory {
    pub player: Pubkey,
    pub ship_id: u64,
    pub width: u8,
    pub height: u8,
    pub next_instance_id: u64,
    // Slot types array (stored as Vec)
    pub slot_types: Vec<SlotType>,
    // Items stored as a flat array for now
    // In production, might want a more efficient structure
    pub items: Vec<GridItem>,
}

impl PlayerInventory {
    pub fn calculate_size(width: u8, height: u8) -> usize {
        8 + // discriminator
        32 + // player
        8 + // ship_id
        1 + // width
        1 + // height
        8 + // next_instance_id
        4 + (width as usize * height as usize) + // slot_types Vec
        4 + (width as usize * height as usize * std::mem::size_of::<GridItem>()) // items Vec
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct GridItem {
    pub item_type: ItemType,
    pub item_id: u64,
    pub instance_id: u64,
    pub rotation: u8,
}

impl Default for GridItem {
    fn default() -> Self {
        Self {
            item_type: ItemType::Empty,
            item_id: 0,
            instance_id: 0,
            rotation: 0,
        }
    }
}

#[account]
pub struct FishCatch {
    pub player: Pubkey,
    pub instance_id: u64,
    pub species: u64,
    pub weight: u16,
    pub caught_timestamp: i64,
}

impl FishCatch {
    pub const LEN: usize = 8 + // discriminator
        32 + // player
        8 + // instance_id
        8 + // species
        2 + // weight
        8; // caught_timestamp
}

// ============ Inventory Module ============

pub mod inventory {
    use super::*;

    pub fn initialize_inventory(
        inventory: &mut Account<PlayerInventory>,
        player: &Pubkey,
        ship_id: u64,
        width: u8,
        height: u8,
        slot_types: Vec<SlotType>,
    ) -> Result<()> {
        require!(width > 0 && height > 0, TidesError::InvalidDimensions);
        require!(slot_types.len() == (width as usize * height as usize), TidesError::ArrayLengthMismatch);

        inventory.player = *player;
        inventory.ship_id = ship_id;
        inventory.width = width;
        inventory.height = height;
        inventory.next_instance_id = 1;
        inventory.slot_types = slot_types;
        
        // Initialize items array with empty items
        let total_slots = (width as usize) * (height as usize);
        inventory.items = vec![GridItem::default(); total_slots];

        Ok(())
    }

    pub fn place_item(
        inventory: &mut Account<PlayerInventory>,
        item_type: ItemType,
        item_id: u64,
        x: u8,
        y: u8,
        rotation: u8,
        shape_width: u8,
        shape_height: u8,
        shape_data: Vec<u8>,
    ) -> Result<u64> {
        require!(rotation < 4, TidesError::InvalidRotation);
        require!(x < inventory.width && y < inventory.height, TidesError::PositionOutOfBounds);

        // Calculate rotated dimensions
        let (final_width, final_height) = if rotation % 2 == 0 {
            (shape_width, shape_height)
        } else {
            (shape_height, shape_width)
        };

        // Check bounds
        require!(
            (x as usize + final_width as usize) <= inventory.width as usize &&
            (y as usize + final_height as usize) <= inventory.height as usize,
            TidesError::PositionOutOfBounds
        );

        // Check if space is available
        for dy in 0..final_height {
            for dx in 0..final_width {
                let idx = coords_to_index(x + dx, y + dy, inventory.width);
                let item = &inventory.items[idx as usize];
                require!(item.item_type == ItemType::Empty, TidesError::PositionOccupied);
                
                let slot_type = inventory.slot_types[idx as usize];
                match item_type {
                    ItemType::Engine => require!(slot_type == SlotType::Engine, TidesError::CannotPlaceItem),
                    ItemType::FishingRod => require!(slot_type == SlotType::FishingRod, TidesError::CannotPlaceItem),
                    _ => require!(slot_type != SlotType::Blocked, TidesError::CannotPlaceItem),
                }
            }
        }

        // Place the item
        let instance_id = inventory.next_instance_id;
        inventory.next_instance_id = inventory
            .next_instance_id
            .checked_add(1)
            .ok_or(TidesError::MathOverflow)?;

        // Simplified placement - in production would check shape data
        for dy in 0..final_height {
            for dx in 0..final_width {
                let idx = coords_to_index(x + dx, y + dy, inventory.width);
                inventory.items[idx as usize] = GridItem {
                    item_type,
                    item_id,
                    instance_id,
                    rotation,
                };
            }
        }

        Ok(instance_id)
    }

    pub fn remove_item(
        inventory: &mut Account<'_, PlayerInventory>,
        instance_id: u64,
    ) -> Result<()> {
        let mut found = false;
        
        for item in inventory.items.iter_mut() {
            if item.instance_id == instance_id {
                *item = GridItem::default();
                found = true;
            }
        }

        require!(found, TidesError::ItemNotFound);

        Ok(())
    }

    pub fn get_item_at(
        inventory: &Account<PlayerInventory>,
        x: u8,
        y: u8,
    ) -> Result<GridItem> {
        require!(x < inventory.width && y < inventory.height, TidesError::PositionOutOfBounds);
        
        let idx = coords_to_index(x, y, inventory.width);
        Ok(inventory.items[idx as usize])
    }

    pub fn has_equipped_item_type(
        inventory: &Account<PlayerInventory>,
        item_type: ItemType,
    ) -> bool {
        inventory.items.iter().any(|item| item.item_type == item_type)
    }

    pub fn coords_to_index(x: u8, y: u8, width: u8) -> u16 {
        (y as u16 * width as u16 + x as u16)
    }
}

pub use inventory::coords_to_index;
