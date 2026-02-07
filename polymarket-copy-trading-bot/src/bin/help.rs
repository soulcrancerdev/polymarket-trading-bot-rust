use polymarket_copy_rust::utils::theme::colors;

fn main() {
    let cyan = colors::ACCENT;
    let green = colors::SUCCESS;
    let yellow = "\x1b[33;1m";
    let blue = "\x1b[34;1m";
    let reset = colors::RESET;

    println!("{cyan}");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("     POLYMARKET COPY TRADING BOT — COMMANDS (Rust)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{reset}\n");

    println!("{yellow}GETTING STARTED{reset}\n");
    println!("  {green}make setup{reset}             Ensure .env exists from .env.example");
    println!("  {green}make health-check{reset}      Verify DB, RPC, balance, API");
    println!("  {green}make build{reset}             Build release binary");
    println!("  {green}make run{reset}               Run the trading bot (default)");
    println!("  {green}make dev{reset}               Run bot in dev (debug build)");
    println!();

    println!("{yellow}WALLET & ALLOWANCE{reset}\n");
    println!("  {green}make check-allowance{reset}    Show USDC balance and Polymarket allowance");
    println!("  {green}make verify-allowance{reset}  Verify allowance (stub)");
    println!("  {green}make set-token-allowance{reset} Set USDC approval (stub)");
    println!("  {green}make check-proxy{reset}        Check proxy wallet (stub)");
    println!("  {green}make check-both{reset}         Check both wallets (stub)");
    println!();

    println!("{yellow}MONITORING & STATS{reset}\n");
    println!("  {green}make check-stats{reset}       Trading stats (stub)");
    println!("  {green}make check-activity{reset}    Recent activity (stub)");
    println!("  {green}make check-pnl{reset}         PnL discrepancy (stub)");
    println!();

    println!("{yellow}POSITION MANAGEMENT{reset}\n");
    println!("  {green}make manual-sell{reset}       Manually sell position (stub)");
    println!("  {green}make sell-large{reset}         Sell large positions (stub)");
    println!("  {green}make close-stale{reset}        Close stale positions (stub)");
    println!("  {green}make close-resolved{reset}     Close resolved (stub)");
    println!("  {green}make redeem-resolved{reset}   Redeem resolved (stub)");
    println!("  {green}make transfer-to-gnosis{reset} Transfer to Gnosis Safe (stub)");
    println!();

    println!("{yellow}TRADER RESEARCH{reset}\n");
    println!("  {green}make find-traders{reset}      Find best traders (stub)");
    println!("  {green}make find-low-risk{reset}      Low-risk traders (stub)");
    println!("  {green}make scan-traders{reset}       Scan traders (stub)");
    println!("  {green}make scan-markets{reset}       Scan from markets (stub)");
    println!();

    println!("{yellow}SIMULATION{reset}\n");
    println!("  {green}make simulate{reset}           Simulate profitability (stub)");
    println!("  {green}make simulate-old{reset}       Old logic sim (stub)");
    println!("  {green}make sim{reset}                 Run simulations (stub)");
    println!("  {green}make compare{reset}            Compare results (stub)");
    println!("  {green}make fetch-history{reset}      Fetch historical trades (stub)");
    println!("  {green}make aggregate{reset}          Aggregate results (stub)");
    println!("  {green}make audit{reset} / make audit-old{reset}  Algorithm audit (stub)");
    println!();

    println!("{blue}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{reset}\n");
    println!("  New user? Run: make setup && make health-check");
    println!("  Then: make run");
    println!("  (Stub) = not yet implemented in Rust; use TS project for full features.)");
    println!();
}
