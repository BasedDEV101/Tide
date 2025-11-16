use anchor_lang::prelude::*;
use anchor_lang::solana_program::keccak;
use crate::TidesError;

// ============ Fishing Constants ============
pub const SIGNATURE_TIMEOUT: i64 = 300; // 5 minutes

// ============ Fishing Account Structures ============

#[account]
pub struct FishingState {
    pub player: Pubkey,
    pub pending_nonce: u64,
    pub bait_type_used: u64,
    pub fishing_nonce: u64,
}

impl FishingState {
    pub const LEN: usize = 8 + // discriminator
        32 + // player
        8 + // pending_nonce
        8 + // bait_type_used
        8; // fishing_nonce
}

#[account]
pub struct PlayerBait {
    pub player: Pubkey,
    pub bait_type: u64,
    pub amount: u64,
}

impl PlayerBait {
    pub const LEN: usize = 8 + // discriminator
        32 + // player
        8 + // bait_type
        8; // amount
}

// ============ Fishing Structs ============

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct FishingResult {
    pub player: Pubkey,
    pub nonce: u64,
    pub species: u64, // 0 = no catch
    pub weight: u16,
    pub timestamp: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct FishPlacement {
    pub should_place: bool, // true = place fish in inventory, false = discard
    pub x: u8, // X coordinate for placement
    pub y: u8, // Y coordinate for placement
    pub rotation: u8, // Rotation: 0=up, 1=right, 2=down, 3=left
}

// ============ Fishing Module ============

pub mod fishing {
    use super::*;

    pub fn initiate_fishing(
        fishing_state: &mut Account<FishingState>,
        bait_type: u64,
        has_fishing_rod: bool,
    ) -> Result<u64> {
        require!(has_fishing_rod, TidesError::NoFishingRodEquipped);
        require!(fishing_state.pending_nonce == 0, TidesError::PendingFishingRequest);

        // Increment fishing nonce
        fishing_state.fishing_nonce = fishing_state
            .fishing_nonce
            .checked_add(1)
            .ok_or(TidesError::MathOverflow)?;

        let fishing_nonce = fishing_state.fishing_nonce;
        fishing_state.pending_nonce = fishing_nonce;
        fishing_state.bait_type_used = bait_type;

        Ok(fishing_nonce)
    }

    pub fn verify_fishing_signature(
        result: &FishingResult,
        signature: &[u8],
        server_signer: &Pubkey,
        clock: &Clock,
    ) -> Result<()> {
        // Verify timestamp is recent
        require!(
            clock.unix_timestamp <= result.timestamp + SIGNATURE_TIMEOUT,
            TidesError::SignatureExpired
        );
        require!(
            result.timestamp <= clock.unix_timestamp,
            TidesError::FutureTimestamp
        );

        // In a full implementation, verify Ed25519 signature
        // For now, this is a placeholder
        // The signature verification would use solana_program::ed25519_program

        Ok(())
    }

    pub fn fulfill_fishing(
        fishing_state: &mut Account<FishingState>,
        result: &FishingResult,
    ) -> Result<()> {
        require!(
            fishing_state.pending_nonce == result.nonce,
            TidesError::ExpiredFishingRequest
        );
        require!(result.nonce > 0, TidesError::InvalidFishingResult);

        // Clear pending request
        fishing_state.pending_nonce = 0;
        fishing_state.bait_type_used = 0;

        Ok(())
    }
}

// Note: Signature verification in Solana uses Ed25519 signatures
// In production, you would use solana_program::ed25519_instruction
// or a similar library for server-side signing verification

