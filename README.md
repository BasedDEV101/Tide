# 🌊 Tides

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/your-repo/tides)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Solana](https://img.shields.io/badge/solana-^1.18-purple.svg)](https://solana.com/)
[![Anchor](https://img.shields.io/badge/anchor-^0.30-red.svg)](https://www.anchor-lang.com/)

An onchain multiplayer fishing game built on the Solana blockchain. Navigate vast oceans, catch rare fish, manage your ship and inventory, and compete in a dynamic player-driven economy.

## 🎮 Game Overview

Tides combines the exploration and resource management of "Dredge" with blockchain technology to create a fully onchain gaming experience. Players explore different maps, catch fish with strategic bait selection, trade in dynamic markets, and climb seasonal leaderboards.

### Core Gameplay Loop
1. **🛒 Preparation** - Buy fuel, bait, and repair equipment
2. **🗺️ Exploration** - Navigate hex-grid oceans across multiple maps  
3. **🎣 Fishing** - Use strategic bait selection to catch rare species
4. **📦 Management** - Organize inventory in Tetris-like cargo system
5. **💰 Trading** - Sell fish in dynamic bonding curve markets
6. **⬆️ Progression** - Upgrade ships and climb leaderboards

## 🏗️ Architecture

### Tech Stack

**Frontend (`app/`)**
- **Framework**: SvelteKit with Svelte 5
- **3D Rendering**: Threlte (Three.js wrapper for Svelte)
- **Styling**: TailwindCSS v4 + bits-ui components
- **Web3**: Solana Web3.js + Anchor for blockchain interactions
- **Deployment**: Cloudflare Pages/Workers

**Smart Contracts (`programs/`)**
- **Framework**: Anchor
- **Language**: Rust
- **Network**: Solana
- **Token**: SPL Token for in-game currency

## 📋 Project Structure

```
tides/
├── 📱 app/                    # SvelteKit frontend
│   ├── src/routes/           # Page routes
│   ├── src/lib/              # Shared components & utilities
│   └── static/               # Static assets
├── ⚖️ programs/               # Solana programs (smart contracts)
│   └── tides/                # Main game program
│       └── src/              # Rust source code
├── 📝 tests/                  # Anchor tests
└── 📄 Anchor.toml            # Anchor configuration
```

## 🚀 Quick Start

### Prerequisites
- **Node.js** 18+ with Bun or npm
- **Solana CLI** toolkit
- **Anchor** framework
- **Rust** (for program development)
- **Git** for version control

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/your-repo/tides.git
   cd tides
   ```

2. **Install frontend dependencies**
   ```bash
   cd app
   bun install
   ```

3. **Install Anchor dependencies**
   ```bash
   anchor build
   ```

### Development

**Frontend Development**
```bash
cd app
bun run dev          # Start development server
bun run build        # Build for production
bun run check        # Type checking
bun run lint         # Lint code
```

**Program Development**
```bash
anchor build          # Build Solana program
anchor test           # Run tests
anchor deploy         # Deploy program
```

## 🎯 Game Mechanics

### 🗺️ Map System
- **Multiple Maps**: Explore different ocean regions with unique fish populations
- **Tiered Content**: Higher-tier maps offer rarer fish but cost more to access
- **Terrain**: Navigate around impassable areas and find optimal fishing spots
- **Travel Costs**: Pay currency to fast-travel between maps

### 🎣 Fishing System
- **Bait Selection**: Choose specific bait types for strategic fishing
- **Position-based**: Fish distributions vary by location on each map
- **Server-Driven**: Off-chain computation with on-chain verification for optimal performance
- **Free Attempts**: No cost per fishing attempt, only bait consumption

### ⛵ Ship & Movement
- **Hex Grid**: Navigate on a hexagonal grid system (6 directions)
- **Fuel Economy**: Movement costs fuel based on distance and ship efficiency
- **Speed System**: `speed = enginePower / totalWeight`
- **Cooldowns**: Movement speed determines how often you can move

### 📦 Inventory Management
- **2D Grid System**: Tetris-like inventory with item shapes
- **Ship Variants**: Different ships have unique cargo hold shapes
- **Equipment Slots**: Designated areas for engines and fishing gear
- **Weight System**: Total weight affects movement speed

### 💰 Dynamic Economy
- **Bonding Curves**: Fish prices change based on supply and demand
- **Freshness Decay**: Fish lose value over time
- **Weight Factor**: Heavier fish are more valuable
- **Market Formula**: `finalPrice = marketValue × weight × freshness`

### 🏆 Seasonal Competition
- **Leaderboards**: Compete based on currency earned minus spent
- **Season Pass**: Purchase passes with SOL to participate
- **Rewards**: Top players earn valuable in-game items
- **Shards**: Multiplayer optimization without affecting game content

## 📄 Solana Programs

### Core Program (`tides`)

The main game program handles:
- Player registration and state management
- Movement and fuel consumption
- Fishing mechanics
- Fish market trading
- Inventory management
- Equipment purchases

### Key Features
- **Map-based World**: Fish distributions and shops unique per map
- **Server-Driven Mechanics**: Off-chain computation with on-chain verification
- **Shard Optimization**: Multiplayer scaling without content fragmentation
- **Production Ready**: Comprehensive error handling and optimization

## 🧪 Testing

**Run All Tests**
```bash
anchor test
```

**Run Specific Test**
```bash
anchor test --skip-local-validator
```

## 🚀 Deployment

### Local Development
```bash
# Start local validator
solana-test-validator

# Deploy program
anchor deploy
```

### Production Deployment
```bash
# Deploy to mainnet-beta or devnet
anchor deploy --provider.cluster mainnet-beta
```

## 🎮 Game Design

Detailed game mechanics and design decisions:
- Core game design and mechanics
- Technical architecture
- Development guidelines

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests and ensure they pass
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

### Development Guidelines
- Follow Rust and Anchor best practices
- Add comprehensive tests for new features
- Update documentation for significant changes
- Use the established code style and conventions

## 📊 Gas Optimization

The program is optimized for Solana:
- **Compact Structs**: Minimize account data size
- **Batch Operations**: Combined actions in single transactions
- **Event-driven**: Minimal on-chain storage for temporary data
- **Efficient Algorithms**: Optimized calculations

## 🔐 Security

- **Access Control**: Role-based permissions for admin functions
- **Reentrancy Protection**: Solana's transaction model prevents reentrancy
- **Input Validation**: Comprehensive parameter checking
- **Signature Verification**: Server-driven mechanics with signature validation
- **Pausable**: Emergency pause functionality

## 📈 Roadmap

### Phase 1: Core Game ✅
- [x] Basic movement and fishing
- [x] Map system with terrain
- [x] Bait selection mechanics
- [x] Server-driven fishing system

### Phase 2: Economy & Trading ✅
- [x] Fish market implementation
- [x] Dynamic pricing system (bonding curves)
- [x] Inventory management (2D grid system)
- [x] Currency and economic mechanics

### Phase 3: Frontend & UI 🚧
- [ ] 3D ocean world with Threlte
- [ ] Hex-grid movement visualization
- [ ] Inventory management UI
- [ ] Solana wallet integration

### Phase 4: Social Features 📋
- [ ] Seasonal leaderboards
- [ ] Guild system
- [ ] Multiplayer events
- [ ] Achievement system

### Phase 5: Advanced Features 📋
- [ ] Equipment crafting
- [ ] Weather system
- [ ] Rare events
- [ ] Enhanced NFT features

## 📞 Support

- **Issues**: Report bugs on GitHub Issues
- **Discussions**: Join community discussions
- **Documentation**: Check project docs for detailed guides

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Solana** for the blockchain infrastructure
- **Anchor** for excellent development tools
- **"Dredge"** for game design inspiration

---

*Built with ❤️ for the future of onchain gaming on Solana*

