# Tides Program Structure

This document outlines the complete structure of the Tides Solana program, converted from the original Ethereum/Solidity contracts.

## 📁 Module Structure

### Core Modules

```
programs/tides/src/
├── lib.rs              # Main program entry point
├── fishing.rs          # Fishing mechanics module
├── inventory.rs        # Inventory management module
├── registries.rs       # Registry contracts (Fish, Ship, Engine, Rod, Map)
└── season_pass.rs      # Season pass and leaderboards module
```

## 🔧 Module Details

### 1. `lib.rs` - Main Program
- **GameState**: Main game state account
- **PlayerState**: Individual player state accounts
- **FishMarketData**: Dynamic fish market pricing
- **ShardData**: Shard management for multiplayer optimization
- Core game functions:
  - `initialize()` - Initialize the game
  - `register_player()` - Register new players
  - `move_player()` - Hex grid movement
  - `purchase_fuel()` - Fuel purchasing
  - `sell_fish()` - Fish market selling

### 2. `fishing.rs` - Fishing Module
- **FishingState**: Player fishing state (nonce, pending requests)
- **PlayerBait**: Bait inventory tracking
- **FishingResult**: Server-signed fishing results
- **FishPlacement**: Fish placement data
- Functions:
  - `initiate_fishing()` - Start fishing with bait
  - `verify_fishing_signature()` - Verify server signatures
  - `fulfill_fishing()` - Complete fishing request

### 3. `inventory.rs` - Inventory Module
- **PlayerInventory**: 2D grid-based inventory system
- **GridItem**: Individual inventory slot item
- **FishCatch**: Fish catch data with freshness tracking
- Functions:
  - `initialize_inventory()` - Setup inventory grid
  - `place_item()` - Place items in inventory (Tetris-like)
  - `remove_item()` - Remove items from inventory
  - `get_item_at()` - Query inventory slot
  - `has_equipped_item_type()` - Check equipment

### 4. `registries.rs` - Registry Modules
All registry contracts for game items:

#### Fish Registry
- **FishRegistry**: Main registry account
- **FishSpecies**: Individual fish species data
- **BaitType**: Bait type definitions

#### Ship Registry
- **ShipRegistry**: Main registry account
- **Ship**: Ship data with cargo dimensions and slot types

#### Engine Registry
- **EngineRegistry**: Main registry account
- **Engine**: Engine stats (power, fuel consumption, shape)

#### Fishing Rod Registry
- **FishingRodRegistry**: Main registry account
- **FishingRod**: Fishing rod data and shape

#### Map Registry
- **MapRegistry**: Main registry account
- **Map**: Map data with boundaries, harbors, travel costs

### 5. `season_pass.rs` - Season Pass Module
- **SeasonPassState**: Season pass system state
- **Season**: Individual season data
- **PlayerSeasonStats**: Player stats per season
- **SeasonLeaderboard**: Leaderboard tracking
- Functions:
  - `create_season()` - Create new season
  - `update_player_stats()` - Update player earnings/spending
  - `is_season_active()` - Check season status
  - `end_season()` - End a season

## 📊 Account Relationships

```
GameState
  ├── PlayerState (1 per player)
  │     ├── PlayerInventory
  │     ├── FishingState
  │     ├── PlayerBait (multiple)
  │     └── FishCatch (multiple)
  │
  ├── ShardData (1 per shard)
  ├── FishMarketData (1 per species)
  │
  └── SeasonPassState
        ├── Season (1 per season)
        ├── PlayerSeasonStats (1 per player per season)
        └── SeasonLeaderboard (1 per season)
```

## 🔐 Access Control

- **Admin**: Full access to admin functions
- **Game Contract**: Can call game-related functions
- **Server Signer**: Signs fishing results (off-chain server)

## 📝 Key Differences from Ethereum Version

1. **Account-Based Storage**: Solana uses separate accounts instead of contract storage
2. **PDA Accounts**: Program Derived Addresses for deterministic account creation
3. **Ed25519 Signatures**: Uses Solana's native signature scheme instead of EIP712
4. **SPL Tokens**: Uses SPL Token standard instead of ERC20
5. **Computation Limits**: Must account for compute units in Solana

## 🚀 Next Steps

1. Implement full fishing instruction handlers
2. Add inventory manipulation instructions
3. Create registry initialization instructions
4. Implement Season Pass NFT minting (using Metaplex)
5. Add comprehensive tests
6. Deploy to devnet

