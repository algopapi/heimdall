# Heimdall

Defi Analytics Engine

## TODOs

### Raydium CP-Swap

**Core Pool Data**

- [ ] Track pool authority and pool account addresses
- [ ] Monitor token mint A & B (addresses, decimals, symbols, metadata)
- [ ] Track pool token mint (LP token) details
- [ ] Monitor fee rates and pool status (active/paused/disabled)
- [ ] Record pool creation block/timestamp and creator wallet
- [ ] Track reserve A & B balances in real-time
- [ ] Calculate and monitor K value (constant product)
- [ ] Track LP token total supply and distribution

**Transaction-Level Data**

- [ ] Capture swap transaction details (signature, user wallet, amounts)
- [ ] Track swap direction (A->B or B->A)
- [ ] Calculate price at execution, slippage tolerance, and actual slippage
- [ ] Monitor price impact and fees paid per transaction
- [ ] Track liquidity addition/removal operations
- [ ] Calculate impermanent loss for LP operations

**MEV & Arbitrage Detection**

- [ ] Implement sandwich attack detection algorithm
- [ ] Track frontrun and backrun transaction patterns
- [ ] Monitor attacker wallets and victim losses
- [ ] Detect cross-DEX arbitrage opportunities
- [ ] Calculate MEV bot profits and market share

**Advanced Analytics**

- [ ] Calculate Total Value Locked (TVL) in real-time
- [ ] Track liquidity depth at different price levels
- [ ] Monitor pool utilization (volume vs liquidity)
- [ ] Implement real-time impermanent loss calculations
- [ ] Detect large trades (>1% of pool size)
- [ ] Monitor price deviation from other exchanges

### Raydium CLMM

**Core Pool Data**

- [ ] Track pool configuration (token pair, fee tier, tick spacing)
- [ ] Monitor current price (sqrt_price_x64) and current tick
- [ ] Track active liquidity at current price
- [ ] Monitor tick array data and tick spacing configuration

**Position NFT Tracking**

- [ ] Track position NFT creation and ownership
- [ ] Monitor position tick lower/upper bounds
- [ ] Track liquidity amount per position
- [ ] Monitor fees accumulated per position
- [ ] Calculate position profitability and IL

**CLMM Transaction Data**

- [ ] Track CLMM-specific swap transactions
- [ ] Monitor liquidity additions with specific tick ranges
- [ ] Track liquidity removals and position adjustments
- [ ] Monitor fee collection transactions

**Advanced CLMM Analytics**

- [ ] Generate liquidity heatmaps by tick range
- [ ] Track most active tick ranges over time
- [ ] Monitor liquidity migration patterns
- [ ] Calculate capital efficiency vs traditional AMM
- [ ] Analyze just-in-time (JIT) liquidity patterns

**CLMM-Specific MEV Detection**

- [ ] Detect JIT liquidity front-running
- [ ] Monitor sandwich attacks in concentrated ranges
- [ ] Track arbitrage in tight liquidity ranges
- [ ] Detect position manipulation attacks
