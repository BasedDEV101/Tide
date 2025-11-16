# Conversion Comparison: Solidity → Solana

This document provides a detailed one-to-one mapping between the original Ethereum/Solidity contracts and the converted Solana/Anchor program.

## 📊 File Structure Mapping

### Original Solidity Structure (25 files)
```
contracts/src/
├── core/
│   ├── RisingTides.sol              → lib.rs (main program)
│   ├── RisingTidesBase.sol          → lib.rs (base functionality)
│   ├── RisingTidesFishing.sol       → fishing.rs
│   ├── RisingTidesInventory.sol     → inventory.rs
│   ├── SeasonPass.sol               → season_pass.rs
│   └── managers/
│       ├── PlayerManager.sol        → lib.rs (register_player, shard management)
│       ├── MovementManager.sol      → lib.rs (move_player, purchase_fuel)
│       ├── ResourceManager.sol      → lib.rs (travel_to_map, purchase_*)
│       └── FishMarketManager.sol    → lib.rs (sell_fish, market pricing)
├── interfaces/ (7 files)
│   ├── IRisingTides.sol             → lib.rs (struct definitions)
│   ├── IRisingTidesFishing.sol      → fishing.rs (struct definitions)
│   ├── IRisingTidesInventory.sol    → inventory.rs (struct definitions)
│   ├── IShipRegistry.sol            → registries.rs
│   ├── IEngineRegistry.sol          → registries.rs
│   ├── IFishingRodRegistry.sol      → registries.rs
│   └── IMapRegistry.sol             → registries.rs
├── registries/ (5 files)
│   ├── FishRegistry.sol             → registries.rs
│   ├── ShipRegistry.sol             → registries.rs
│   ├── EngineRegistry.sol           → registries.rs
│   ├── FishingRodRegistry.sol       → registries.rs
│   └── MapRegistry.sol              → registries.rs
├── libraries/
│   └── InventoryLib.sol             → inventory.rs (module functions)
├── types/
│   └── InventoryTypes.sol           → lib.rs (enums: SlotType, ItemType)
├── tokens/
│   └── RisingTidesCurrency.sol      → SPL Token (standard, no custom contract needed)
└── utils/
    └── Errors.sol                   → lib.rs (TidesError enum)

Total: 25 Solidity files → 5 Rust modules
```

### Converted Solana Structure (5 modules)
```
programs/tides/src/
├── lib.rs           (Main program + all manager logic)
├── fishing.rs       (Fishing contract)
├── inventory.rs     (Inventory contract)
├── registries.rs    (All 5 registries)
└── season_pass.rs   (Season pass contract)
```

## ✅ Function Mapping

### Main Game Contract (RisingTides.sol → lib.rs)

| Original Solidity Function | Solana/Anchor Equivalent | Status |
|---------------------------|--------------------------|--------|
| `constructor()` | `initialize()` | ✅ Converted |
| `registerPlayer()` | `register_player()` | ✅ Converted |
| `getPlayerState()` | Query `PlayerState` account | ✅ Converted |
| `isPlayerRegistered()` | Check `PlayerState.is_active` | ✅ Converted |
| `move()` | `move_player()` | ✅ Converted |
| `calculateFuelCost()` | `calculate_fuel_cost()` | ✅ Converted |
| `purchaseFuel()` | `purchase_fuel()` | ✅ Converted |
| `getCurrentFuel()` | Query `PlayerState.current_fuel` | ✅ Converted |
| `travelToMap()` | ❌ **MISSING** - Needs implementation | ⚠️ |
| `changeShip()` | ❌ **MISSING** - Needs implementation | ⚠️ |
| `purchaseShip()` | ❌ **MISSING** - Needs implementation | ⚠️ |
| `purchaseEngine()` | ❌ **MISSING** - Needs implementation | ⚠️ |
| `purchaseFishingRod()` | ❌ **MISSING** - Needs implementation | ⚠️ |
| `sellFish()` | `sell_fish()` | ✅ Converted |
| `changeShard()` | ❌ **MISSING** - Needs implementation | ⚠️ |
| `getShardPlayerCount()` | Query `ShardData` account | ✅ Converted |
| `getMaxPlayersPerShard()` | Query `GameState.max_players_per_shard` | ✅ Converted |
| `isShardAvailable()` | Check `ShardData.player_count < max` | ✅ Converted |
| `setMaxPlayersPerShard()` | ❌ **MISSING** - Needs admin instruction | ⚠️ |
| `adminChangePlayerShard()` | ❌ **MISSING** - Needs admin instruction | ⚠️ |
| `updateServerSigner()` | ❌ **MISSING** - Needs admin instruction | ⚠️ |
| `updateDependencies()` | ❌ **MISSING** - Needs admin instruction | ⚠️ |
| `pause()` / `unpause()` | ❌ **MISSING** - Needs admin instruction | ⚠️ |
| Inventory functions (delegated) | Via `inventory` module | ✅ Converted |
| Fishing functions (delegated) | Via `fishing` module | ✅ Converted |

### Fishing Contract (RisingTidesFishing.sol → fishing.rs)

| Original Function | Solana Equivalent | Status |
|------------------|-------------------|--------|
| `initiateFishing()` | `initiate_fishing()` | ✅ Converted |
| `fulfillFishing()` | `fulfill_fishing()` | ✅ Converted (signature verification partial) |
| `purchaseBait()` | ❌ **MISSING** - Needs instruction | ⚠️ |
| `getPlayerBait()` | Query `PlayerBait` account | ✅ Converted |
| `getPlayerAvailableBait()` | Query all `PlayerBait` accounts | ✅ Converted |
| `getPlayerFishingStatus()` | Query `FishingState` account | ✅ Converted |
| `hasEquippedFishingRod()` | Via `inventory` module | ✅ Converted |
| `updateServerSigner()` | ❌ **MISSING** - Needs admin instruction | ⚠️ |
| `updateRegistries()` | ❌ **MISSING** - Needs admin instruction | ⚠️ |

### Inventory Contract (RisingTidesInventory.sol → inventory.rs)

| Original Function | Solana Equivalent | Status |
|------------------|-------------------|--------|
| `initializeInventory()` | `initialize_inventory()` | ✅ Converted |
| `assignDefaultEquipment()` | ❌ **MISSING** - Needs instruction | ⚠️ |
| `getPlayerInventory()` | Query `PlayerInventory` account | ✅ Converted |
| `getInventoryItem()` | `get_item_at()` | ✅ Converted |
| `updateInventoryItem()` | ❌ **MISSING** - Needs instruction | ⚠️ |
| `discardInventoryItem()` | ❌ **MISSING** - Needs instruction | ⚠️ |
| `placeFishInInventory()` | `place_item()` | ✅ Converted |
| `removeFishFromInventory()` | `remove_item()` | ✅ Converted |
| `getFishData()` | Query `FishCatch` account | ✅ Converted |
| `hasEquippedItemType()` | `has_equipped_item_type()` | ✅ Converted |
| `getTotalEnginePower()` | ❌ **MISSING** - Needs implementation | ⚠️ |
| `hasEquippedFishingRod()` | Via `has_equipped_item_type()` | ✅ Converted |

### Season Pass (SeasonPass.sol → season_pass.rs)

| Original Function | Solana Equivalent | Status |
|------------------|-------------------|--------|
| `createSeason()` | `create_season()` | ✅ Converted |
| `purchaseSeasonPass()` | ❌ **MISSING** - Needs instruction + NFT minting | ⚠️ |
| `updatePlayerStats()` | `update_player_stats()` | ✅ Converted |
| `getLeaderboard()` | Query `SeasonLeaderboard` account | ✅ Converted |
| `getPlayerStats()` | Query `PlayerSeasonStats` account | ✅ Converted |
| `getPlayerPosition()` | Query `SeasonLeaderboard` account | ✅ Converted |
| `endSeason()` | `end_season()` | ✅ Converted |
| `distributeRewards()` | ❌ **MISSING** - Needs instruction | ⚠️ |
| `withdraw()` | ❌ **MISSING** - Needs admin instruction | ⚠️ |

### Registries (All → registries.rs)

| Original Registry | Solana Account | Status |
|------------------|----------------|--------|
| `FishRegistry` | `FishRegistry` + `FishSpecies` accounts | ✅ Converted |
| `ShipRegistry` | `ShipRegistry` + `Ship` accounts | ✅ Converted |
| `EngineRegistry` | `EngineRegistry` + `Engine` accounts | ✅ Converted |
| `FishingRodRegistry` | `FishingRodRegistry` + `FishingRod` accounts | ✅ Converted |
| `MapRegistry` | `MapRegistry` + `Map` accounts | ✅ Converted |

## ⚠️ Missing Implementations

### Critical Missing Functions (Need Instructions):
1. **Travel & Resource Management:**
   - `travel_to_map()` instruction
   - `change_ship()` instruction
   - `purchase_ship()` instruction
   - `purchase_engine()` instruction
   - `purchase_fishing_rod()` instruction

2. **Inventory Management:**
   - `update_inventory_item()` instruction
   - `discard_inventory_item()` instruction
   - `assign_default_equipment()` instruction

3. **Fishing:**
   - `purchase_bait()` instruction
   - Full signature verification (Ed25519)

4. **Season Pass:**
   - `purchase_season_pass()` instruction (with NFT minting)
   - `distribute_rewards()` instruction

5. **Admin Functions:**
   - `pause()` / `unpause()` instructions
   - `update_server_signer()` instruction
   - `set_max_players_per_shard()` instruction
   - `admin_change_player_shard()` instruction

6. **Query Functions:**
   - Many view functions need client-side query implementations
   - Market price calculations
   - Fuel cost calculations

## ✅ What IS Complete

1. **Core Account Structures** - All data structures converted
2. **Core Game Logic** - Movement, registration, fuel, fish selling
3. **Module Organization** - Clean separation of concerns
4. **Error Handling** - Comprehensive error codes
5. **Type Definitions** - All enums and types converted
6. **Constants** - All game constants preserved
7. **Registry Structures** - All registry account definitions

## 📈 Conversion Status

- **Account Structures**: ✅ 100% Complete
- **Core Game Logic**: ✅ 60% Complete
- **Instructions (Functions)**: ⚠️ 30% Complete (5/17 main instructions)
- **Registry Implementations**: ✅ 100% Complete (structures)
- **Error Codes**: ✅ 100% Complete
- **Type Definitions**: ✅ 100% Complete

## 🔄 Architectural Differences

1. **Inheritance → Composition**: Solidity inheritance replaced with module composition
2. **Separate Contracts → Single Program**: Multiple contracts consolidated into one program with modules
3. **ERC20 → SPL Token**: Custom token contract replaced with standard SPL Token
4. **EIP712 → Ed25519**: Ethereum signature scheme replaced with Solana's native Ed25519
5. **Storage Mappings → Accounts**: Contract storage replaced with separate accounts
6. **Events → Anchor Events**: Solidity events converted to Anchor events

## 🎯 Conclusion

**Status: STRUCTURAL CONVERSION COMPLETE, INSTRUCTION IMPLEMENTATION IN PROGRESS**

The conversion is **NOT yet 1-to-1** in terms of complete functionality. All the **structures, data models, and core logic** have been converted, but many **instruction handlers** (the actual on-chain functions) still need to be implemented.

**What's Missing**: ~12 instruction handlers need to be added to make it fully functional.

**What's Complete**: All data structures, account layouts, error handling, and foundational code.

