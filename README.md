# Solana Ultra-Fast New Token Sniper on Raydium and Pump.fun in Rust 🚀

## Overview

Introducing the **Solana Ultra-Fast New Token Sniper** written in **Rust** 🦀, designed to detect newly launched tokens on both **Raydium** and **Pump.fun** and execute buys at lightning speed ⚡. With Rust's memory safety features and performance optimizations, built with Rust for superior performance and security. Tailored for both novice and experienced traders.

---

## Key Features

### 🚀 Speed and Efficiency
- **Lightning-Quick Transactions**: Leveraging Rust's exceptional performance, our bot allows you to snipe new tokens almost instantly. Say goodbye to delays and seize opportunities as they arise!

### 🔒 Safety First
- **Robust Security**: Rust's safety guarantees minimize bugs and vulnerabilities, ensuring your trading activities are secure. Trade with confidence and peace of mind.

### 📊 Multiple gRPC Connections
- **Stay Updated**: Effortlessly connect to top Solana data providers like **Helius** and **Yellowstone** through multiple gRPC connections. Get real-time updates and make informed trading decisions.

### 👩‍💻 User-Friendly Interface
- **Intuitive Design**: Our sniper bot features a clean and accessible interface, making it easy for users of all experience levels to navigate. Start trading in no time!

### 🛠️ Rich Utilities
- **Advanced Features**:
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

### **Solana PumpRay Sniper (Trial)**  
> 🗂️ [solana-pumpray-sniper(trial).zip](https://github.com/user-attachments/files/19416260/solana-pumpray-sniper.trial.zip)

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
3. Run `solana-pumpray-sniper.exe`.

![image](https://github.com/user-attachments/assets/dffc8e4b-cd00-4921-8488-e25230f4a31a)

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
## Support

As the experimental result, the best environment for running this bot is to use `dedicated server located in NY`. 
For support and further inquiries, please connect via Telegram: 📞 [soulcrancerdev](https://t.me/soulcrancerdev).
