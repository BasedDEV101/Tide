# Migration Guide: Rising Tides to Tides (Ethereum to Solana)

This document outlines the migration from the Ethereum/Solidity version to the Solana/Anchor version.

## Overview

The game has been fully converted from:
- **Ethereum/Solidity** → **Solana/Anchor (Rust)**
- **Viem/Wagmi** → **Solana Web3.js + Anchor**
- **Rising Tides** → **Tides**

## Key Changes

### Smart Contracts → Solana Programs

1. **Solidity Contracts** → **Anchor Programs (Rust)**
   - `RisingTides.sol` → `programs/tides/src/lib.rs`
   - All Solidity contracts consolidated into a single Anchor program
   - Contract inheritance replaced with Anchor account structures

2. **Token Contract**
   - `RisingTidesCurrency.sol` → SPL Token (standard Solana token)
   - ERC20 token → SPL Token with mint/burn capabilities

3. **Account Structure**
   - Ethereum `mapping` → Solana `Account` structures
   - Storage optimized for Solana's account model
   - PDAs (Program Derived Addresses) for deterministic accounts

### Frontend Changes

1. **Web3 Libraries**
   - Removed: `viem`, `wagmi`
   - Added: `@solana/web3.js`, `@coral-xyz/anchor`, `@solana/wallet-adapter-*`

2. **Wallet Integration**
   - Ethereum wallet connection → Solana wallet adapter
   - MetaMask → Phantom/Solflare wallets

3. **Transaction Handling**
   - Ethereum transactions → Solana transactions
   - Different fee model (gas → transaction fees)

### Naming Changes

All instances of:
- `rising-tides` → `tides`
- `RisingTides` → `Tides`
- `RTC` (Rising Tides Coin) → `TTC` (Tides Token)

## Architecture Differences

### Ethereum Model
- Contracts deployed to addresses
- State stored in contract storage
- Gas fees per operation
- Sequential transaction processing

### Solana Model
- Programs deployed with program IDs
- State stored in separate accounts
- Low transaction fees
- Parallel transaction processing

## Remaining Work

### Registry Contracts
The registry contracts (Ship, Fish, Engine, FishingRod, Map) need to be fully implemented:
- These can be stored as accounts or PDAs
- Registry data can be stored on-chain or referenced off-chain
- Full implementation depends on design decisions

### Complete Features
- Full fishing mechanics implementation
- Complete inventory management
- Map registry system
- Equipment registries

## Development Setup

### Solana Development
```bash
# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# Install Anchor
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest

# Build program
anchor build

# Test
anchor test
```

### Frontend Development
```bash
cd app
bun install
bun run dev
```

## Notes

- The core game logic has been converted
- Account structures are optimized for Solana
- Error handling follows Solana/Anchor patterns
- Events are properly defined for Solana
- The program ID is placeholder - update before deployment

## Resources

- [Solana Documentation](https://docs.solana.com/)
- [Anchor Documentation](https://www.anchor-lang.com/)
- [Solana Cookbook](https://solanacookbook.com/)

