# Polymarket Arbitrage Bot (Rust)

<div align="center">

**Real-time arbitrage detection and automatic trading bot for Polymarket's 15-minute crypto markets**

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-ISC-yellow.svg)](LICENSE)

</div>

---

## 📋 Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Configuration](#configuration)
- [Usage](#usage)
- [Architecture](#architecture)
- [Troubleshooting](#troubleshooting)
- [Security](#security)
- [License](#license)

---

## 🎯 Overview

The **Polymarket Arbitrage Bot (Rust)** is a high-performance trading bot designed to detect and exploit arbitrage opportunities in Polymarket's 15-minute cryptocurrency markets. The bot monitors real-time orderbook data via WebSocket connections and automatically executes trades when profitable opportunities arise.

### How It Works

The bot identifies arbitrage opportunities when the sum of the best ask prices for UP and DOWN tokens is less than 1.0:

```
UP_ASK + DOWN_ASK < 1.0
```

When this condition is met, the bot simultaneously purchases both tokens at the same USDC amount, locking in a guaranteed profit when the market resolves (since UP + DOWN must equal 1.0 at resolution).

---

## ✨ Features

### Core Capabilities

- 🔍 **Real-time Market Monitoring**: WebSocket-based orderbook updates for instant price discovery
- ⚡ **Automatic Arbitrage Detection**: Detects opportunities when `UP_ASK + DOWN_ASK < 1.0`
- 🤖 **Automated Trading**: Executes simultaneous buy orders for both UP and DOWN tokens
- 💰 **Configurable Trade Size**: Set custom USDC amount per token via environment variables
- 📊 **Interactive Terminal UI**: User-friendly interface with arrow key navigation
- 📈 **Price History Display**: Shows last 10 price updates with timestamps
- 🎨 **Color-Coded Output**: Enhanced readability with colored terminal output
- 🔒 **Duplicate Prevention**: Tracks recent opportunities to avoid redundant trades
- ⚙️ **Smart Wallet Detection**: Automatically detects Gnosis Safe vs EOA wallet types

### Supported Markets

- **BTC** (Bitcoin)
- **ETH** (Ethereum)
- **SOL** (Solana)
- **XRP** (Ripple)

All markets are 15-minute prediction markets on Polymarket.

---

## 📦 Prerequisites

Before installing and running the bot, ensure you have the following:

- **Rust** (1.70 or higher) - [Install Rust](https://www.rust-lang.org/tools/install)
- **Polymarket Account** with:
  - Private key for wallet authentication
  - Proxy wallet address (optional, for Gnosis Safe users)
  - Sufficient USDC balance for trading
- **Polygon Network** RPC endpoint access

---

## 🚀 Installation

### 1. Clone the Repository

```bash
git clone <repository-url>
cd arb-rust
```

### 2. Build the Project

```bash
cargo build --release
```

---

## ⚙️ Configuration

### Environment Variables

Create a `.env` file in the project root with the following variables:

```env
# Required: Wallet Configuration
PRIVATE_KEY=your_wallet_private_key_here
PROXY_WALLET=your_proxy_wallet_address_here

# Optional: Trading Configuration
ARBITRAGE_AMOUNT_USDC=1.0
ARBITRAGE_THRESHOLD=1.0
TOKEN_AMOUNT=5.0

# Optional: API Endpoints (defaults provided)
CLOB_HTTP_URL=https://clob.polymarket.com
CLOB_WS_URL=wss://ws-subscriptions-clob.polymarket.com/ws/market
RPC_URL=https://polygon-rpc.com
USDC_CONTRACT_ADDRESS=0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174
```

### Configuration Details

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `PRIVATE_KEY` | ✅ Yes | - | Your wallet's private key (without `0x` prefix) |
| `PROXY_WALLET` | ✅ Yes | - | Your proxy wallet or main wallet address |
| `ARBITRAGE_AMOUNT_USDC` | ❌ No | `1.0` | USDC amount to spend per token (UP and DOWN) |
| `ARBITRAGE_THRESHOLD` | ❌ No | `1.0` | Threshold for arbitrage detection |
| `TOKEN_AMOUNT` | ❌ No | `5.0` | Fixed token amount to buy for each side |
| `CLOB_HTTP_URL` | ❌ No | `https://clob.polymarket.com` | Polymarket CLOB HTTP API endpoint |
| `CLOB_WS_URL` | ❌ No | `wss://...` | Polymarket WebSocket endpoint |
| `RPC_URL` | ❌ No | `https://polygon-rpc.com` | Polygon network RPC endpoint |
| `USDC_CONTRACT_ADDRESS` | ❌ No | `0x2791...` | USDC contract address on Polygon |

### Security Note

⚠️ **Never commit your `.env` file to version control.** The `.gitignore` file is already configured to exclude it.

---

## 🎮 Usage

### Development Mode

Run the bot in development mode:

```bash
cargo run
```

### Release Mode

Build and run the optimized release version:

```bash
cargo build --release
./target/release/arb-rust
```

### Interactive Interface

Once started, the bot will:

1. **Display Coin Selection Menu**: Use ↑/↓ arrow keys to navigate
2. **Select a Coin**: Press Enter to select BTC, ETH, SOL, or XRP
3. **View Market Data**: See real-time price updates and 10-line history
4. **Monitor Arbitrage**: Detected opportunities are logged and executed automatically

### Keyboard Controls

- **↑/↓ Arrow Keys**: Navigate coin selection menu
- **Enter**: Select coin / Return to menu
- **Ctrl+C**: Exit the bot gracefully

---

## 🏗️ Architecture

### Project Structure

```
arb-rust/
├── src/
│   ├── config/
│   │   ├── constants.rs      # Trading and API constants
│   │   └── env.rs            # Environment variable configuration
│   ├── services/
│   │   ├── create_clob_client.rs # ClobClient initialization and authentication
│   │   ├── arbitrage_executor.rs # Trade execution logic
│   │   ├── market_discovery.rs   # Market discovery for 15-minute markets
│   │   ├── price_monitor.rs      # Price data management and display
│   │   └── websocket_client.rs   # WebSocket client for real-time updates
│   ├── utils/
│   │   ├── keyboard.rs       # Keyboard input handling
│   │   ├── coin_selector.rs  # Coin selection UI
│   │   └── logger.rs         # Logging utilities
│   └── main.rs               # Main entry point
├── .env                      # Environment variables (not committed)
├── .gitignore
├── Cargo.toml
└── README.md
```

---

## 🔒 Security

### Best Practices

1. **Private Key Protection**
   - Never share or commit your private key
   - Use environment variables (not hardcoded values)
   - Consider using a dedicated trading wallet with limited funds

2. **Proxy Wallet Setup**
   - Use Gnosis Safe or similar multisig wallet for enhanced security
   - Regularly review and update wallet permissions
   - Monitor wallet activity for suspicious transactions

3. **Network Security**
   - Use trusted RPC endpoints
   - Verify WebSocket connections are to official Polymarket endpoints
   - Consider using VPN for additional security

### Disclaimer

⚠️ **Trading cryptocurrencies and prediction markets involves risk. Use this bot at your own discretion. The authors are not responsible for any financial losses.**

---

## 📝 License

This project is licensed under the ISC License.

---

<div align="center">

**Built with ❤️ for the Polymarket community**

⭐ Star this repo if you find it useful!

</div>

