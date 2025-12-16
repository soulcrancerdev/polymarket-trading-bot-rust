# Multi-Chain Prediction Market Trading System

A unified Python framework for trading across multiple prediction market platforms on different blockchains. This project provides a single interface to interact with Polymarket, Augur, Moonopol, Myriad Markets, Drift BET, O.LAB, Polkamarkets, and Hedgehog Markets.

## Features

- **Multi-Blockchain Support**: Trade on Ethereum, Polygon, Solana, and BNB Chain
- **Unified Interface**: Single API to interact with all supported prediction markets
- **Platform Support**:
  - **Polymarket** (Polygon)
  - **Augur** (Ethereum)
  - **Moonopol** (Solana)
  - **Myriad Markets** (Ethereum/Polygon)
  - **Drift BET** (Solana)
  - **O.LAB** (BNB Chain)
  - **Polkamarkets** (Polygon)
  - **Hedgehog Markets** (Ethereum)

- **Core Functionality**:
  - Fetch markets and market data
  - Get orderbooks and prices
  - Create and close positions
  - Check balances across chains
  - Monitor positions across platforms

## Architecture

The project follows a modular architecture with clear separation of concerns:

```
src/
├── core/              # Core interfaces and data models
│   └── interfaces.py  # Abstract base classes and data structures
├── blockchains/       # Blockchain connectors
│   ├── ethereum.py   # Ethereum connector
│   ├── polygon.py    # Polygon connector (EVM-compatible)
│   ├── solana.py     # Solana connector
│   └── bnb_chain.py  # BNB Chain connector (EVM-compatible)
├── markets/          # Prediction market platform connectors
│   ├── polymarket.py
│   ├── augur.py
│   ├── moonopol.py
│   ├── myriad.py
│   ├── drift.py
│   ├── olab.py
│   ├── polkamarkets.py
│   └── hedgehog.py
├── trader/           # Unified trading orchestrator
│   └── orchestrator.py
└── utils/            # Utilities
    └── config.py     # Configuration management
```

## Installation

1. **Clone the repository**:
```bash
git clone <repository-url>
cd git
```

2. **Create a virtual environment** (recommended):
```bash
python -m venv venv

# On Windows
venv\Scripts\activate

# On Linux/Mac
source venv/bin/activate
```

3. **Install dependencies**:
```bash
pip install -r requirements.txt
```

4. **Set up environment variables**:
```bash
# Copy the example environment file
cp .env.example .env

# Edit .env with your RPC URLs and private keys
```

## Configuration

Create a `.env` file in the project root with the following variables:

```env
# Ethereum Configuration
ETHEREUM_RPC_URL=https://mainnet.infura.io/v3/YOUR_PROJECT_ID
ETHEREUM_PRIVATE_KEY=your_ethereum_private_key_here

# Polygon Configuration
POLYGON_RPC_URL=https://polygon-rpc.com
POLYGON_PRIVATE_KEY=your_polygon_private_key_here

# Solana Configuration
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
SOLANA_PRIVATE_KEY=your_base58_encoded_solana_private_key_here

# BNB Chain Configuration
BNB_RPC_URL=https://bsc-dataseed.binance.org
BNB_PRIVATE_KEY=your_bnb_chain_private_key_here
```

**Security Note**: Never commit your `.env` file or private keys to version control. The `.gitignore` file is configured to exclude `.env` files.

### Getting RPC URLs

- **Ethereum**: Use Infura, Alchemy, or QuickNode
- **Polygon**: Use Polygon RPC endpoints or Alchemy
- **Solana**: Use public RPC endpoints or Helius, QuickNode
- **BNB Chain**: Use Binance public RPC or QuickNode

## Support

For issues, questions, or contributions, please open an issue on the repository or 📞 [soulcrancerdev](https://t.me/soulcrancerdev).
