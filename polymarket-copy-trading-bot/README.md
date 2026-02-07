# Polymarket Copy Trading Bot

Rust-based bot that automatically copies trades from tracked Polymarket traders to your wallet.

## Quick Start

1. **Install Rust** (if needed): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

2. **Clone & Build**:
   ```bash
   git clone <repo-url>
   cd poly-copy-tg-v1.0
   cargo build --release
   ```

3. **Configure**:
   - Copy `.config.example` to `.env` or create user config
   - Fill in required settings (trader addresses, wallet, private key, RPC URL)

4. **Run**:
   ```bash
   cargo run --release
   ```

## Required Config

- `USER_ADDRESSES` - Comma-separated trader addresses to copy
- `PROXY_WALLET` - Your wallet address
- `PRIVATE_KEY` - Your wallet's private key (64-char hex, no 0x)
- `RPC_URL` - Polygon RPC endpoint
- `MONGO_URI` - MongoDB connection (optional, defaults to localhost)

## Features

- **Real-time monitoring** via RTDS WebSocket
- **Multiple strategies**: Percentage, Fixed, or Adaptive copy sizes
- **Trade aggregation** for small trades
- **Position tracking** in MongoDB
- **Telegram bot** for remote control (optional)

## Commands

- `cargo run --bin health_check` - Check system status
- `cargo run --bin validate_setup` - Validate config
- `cargo run --bin check_allowance` - Check USDC allowance
- `cargo run --bin check_stats` - View trading stats
- `cargo run --bin telegram_bot` - Start Telegram bot

## Setup Token Allowance

Before trading, approve USDC spending:
```bash
cargo run --bin set_token_allowance
```

## Notes

- Works on Polygon network
- Supports EOA and Gnosis Safe wallets
- Trades execute via Polymarket CLOB API
- Keep your private key secure!

