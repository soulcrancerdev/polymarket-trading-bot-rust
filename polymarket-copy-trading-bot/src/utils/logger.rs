use std::fs::OpenOptions;
use std::io::Write;

use super::theme::{self, colors, icons};

pub struct Logger;

impl Logger {
    fn log_dir() -> std::path::PathBuf {
        std::env::current_dir().unwrap_or_default().join("logs")
    }

    fn log_file() -> std::path::PathBuf {
        let date = chrono::Utc::now().format("%Y-%m-%d");
        Self::log_dir().join(format!("bot-{}.log", date))
    }

    fn ensure_log_dir() {
        let dir = Self::log_dir();
        if !dir.exists() {
            let _ = std::fs::create_dir_all(&dir);
        }
    }

    fn write_file(msg: &str) {
        Self::ensure_log_dir();
        if let Ok(mut f) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(Self::log_file())
        {
            let _ = writeln!(f, "[{}] {}", chrono::Utc::now().to_rfc3339(), msg);
        }
    }

    pub fn info(msg: &str) {
        println!(
            "{} {} {}{} {}",
            colors::ACCENT,
            icons::INFO,
            colors::RESET,
            colors::MUTED,
            msg
        );
        let _ = std::io::stdout().flush();
        Self::write_file(&format!("INFO: {}", msg));
    }

    pub fn success(msg: &str) {
        println!(
            "{} {} {}{} {}",
            colors::SUCCESS,
            icons::OK,
            colors::RESET,
            colors::MINT,
            msg
        );
        let _ = std::io::stdout().flush();
        Self::write_file(&format!("SUCCESS: {}", msg));
    }

    pub fn warning(msg: &str) {
        eprintln!(
            "{} {} {}{} {}",
            colors::WARN,
            icons::WARN,
            colors::RESET,
            colors::WARN,
            msg
        );
        let _ = std::io::stderr().flush();
        Self::write_file(&format!("WARNING: {}", msg));
    }

    pub fn error(msg: &str) {
        eprintln!(
            "{} {} {}{} {}",
            colors::ERROR,
            icons::ERR,
            colors::RESET,
            colors::ERROR,
            msg
        );
        let _ = std::io::stderr().flush();
        Self::write_file(&format!("ERROR: {}", msg));
    }

    pub fn separator() {
        println!("{}{} {}", colors::DIM, "─".repeat(72), colors::RESET);
        let _ = std::io::stdout().flush();
    }

    pub fn header(title: &str) {
        let width = 70usize;
        let pad_left = (width - 2 - title.len()) / 2;
        let pad_right = width - 2 - title.len() - pad_left;
        let title_line = format!(
            "{}│{}{}{}{}{}│{}",
            colors::BOX,
            colors::RESET,
            " ".repeat(pad_left),
            format!("{}{}{}", colors::ACCENT_BOLD, title, colors::RESET),
            " ".repeat(pad_right),
            colors::BOX,
            colors::RESET
        );
        println!();
        println!("{}", theme::panel_top(width));
        println!("{}", title_line);
        println!("{}", theme::panel_bottom(width));
        println!();
        let _ = std::io::stdout().flush();
        Self::write_file(&format!("HEADER: {}", title));
    }

    pub fn format_address(addr: &str) -> String {
        let s = addr.trim_start_matches("0x");
        if s.len() >= 10 {
            format!("0x{}…{}", &s[..6], &s[s.len() - 4..])
        } else {
            addr.to_string()
        }
    }

    pub fn startup(traders: &[String], my_wallet: &str) {
        println!();
        for (i, line) in theme::BANNER.iter().enumerate() {
            let color = if i < 3 {
                colors::ACCENT
            } else {
                colors::HIGHLIGHT
            };
            println!("{}{}{}", color, line, colors::RESET);
        }
        println!(
            "{}  {} Copy the best. Automate success.{}",
            colors::MUTED,
            icons::ARROW,
            colors::RESET
        );
        println!();
        let _ = std::io::stdout().flush();

        let width = 68;
        println!("{}", theme::panel_top(width));
        println!(
            "{}│{}  {} Alpha sources ({}) {}│{}",
            colors::BOX,
            colors::RESET,
            colors::ACCENT,
            traders.len(),
            colors::RESET,
            colors::BOX
        );
        for (i, addr) in traders.iter().enumerate() {
            println!(
                "{}│{}    {}. {} {}│{}{}",
                colors::BOX,
                colors::RESET,
                i + 1,
                Self::format_address(addr),
                colors::MUTED,
                colors::BOX,
                colors::RESET
            );
        }
        let masked = if my_wallet.len() >= 42 {
            format!(
                "{}•••{}",
                &my_wallet[..6],
                &my_wallet[my_wallet.len() - 4..]
            )
        } else {
            my_wallet.to_string()
        };
        println!(
            "{}│{}  {} Your vault {} {}│{}{}",
            colors::BOX,
            colors::RESET,
            colors::HIGHLIGHT,
            masked,
            colors::RESET,
            colors::BOX,
            colors::RESET
        );
        println!("{}", theme::panel_bottom(width));
        println!();
        let _ = std::io::stdout().flush();
    }

    pub fn waiting(trader_count: usize, extra: Option<&str>) {
        let ts = chrono::Local::now().format("%H:%M:%S");
        let msg = match extra {
            Some(e) => format!(
                "{} Listening for signals from {} alpha sources {} ({})",
                icons::ARROW,
                trader_count,
                colors::DIM,
                e
            ),
            None => format!(
                "{} Listening for signals from {} alpha sources…",
                icons::ARROW,
                trader_count
            ),
        };
        print!("\r{}[{}] {}{}  ", colors::MUTED, ts, msg, colors::RESET);
        let _ = std::io::stdout().flush();
    }

    pub fn clear_line() {
        print!("\r{}\r", " ".repeat(100));
        let _ = std::io::stdout().flush();
    }

    pub fn money(amount: f64) -> String {
        format!("{}$ {:.2}{}", colors::GOLD, amount, colors::RESET)
    }

    pub fn field(label: &str, value: &str) {
        println!("  {} {} {} {}", colors::MUTED, label, colors::ACCENT, value);
        let _ = std::io::stdout().flush();
    }

    pub fn health_line(label: &str, status: &str, message: &str) {
        let (icon, color) = match status {
            "ok" => (icons::OK, colors::SUCCESS),
            "warning" => (icons::WARN, colors::WARN),
            _ => (icons::ERR, colors::ERROR),
        };
        println!(
            "  {} {} {} {} {} {} {}",
            color,
            icon,
            colors::RESET,
            colors::MUTED,
            label,
            colors::RESET,
            message
        );
        let _ = std::io::stdout().flush();
    }

    pub fn trade(trader_address: &str, action: &str, details: TradeDetails) {
        println!();
        println!("{}{}", colors::HIGHLIGHT, "─".repeat(70));
        println!("{}📊 NEW TRADE DETECTED{}", format!("{}{}", colors::HIGHLIGHT, colors::BOLD), colors::RESET);
        println!("{}Trader: {}{}", colors::MUTED, Self::format_address(trader_address), colors::RESET);
        println!("{}Action: {}{}{}", colors::MUTED, colors::RESET, action, colors::RESET);
        if let Some(asset) = &details.asset {
            println!("{}Asset:  {}{}", colors::MUTED, Self::format_address(asset), colors::RESET);
        }
        if let Some(side) = &details.side {
            let side_color = if side == "BUY" { colors::SUCCESS } else { colors::ERROR };
            println!("{}Side:   {}{}{}{}", colors::MUTED, side_color, colors::BOLD, side, colors::RESET);
        }
        if let Some(amount) = details.amount {
            println!("{}Amount: {}$ {:.2}{}", colors::MUTED, colors::WARN, amount, colors::RESET);
        }
        if let Some(price) = details.price {
            println!("{}Price:  {}{}{}", colors::MUTED, colors::ACCENT, price, colors::RESET);
        }
        if let Some(slug) = details.event_slug.or(details.slug) {
            let market_url = format!("https://polymarket.com/event/{}", slug);
            println!("{}Market: {}{}{}", colors::MUTED, colors::ACCENT, market_url, colors::RESET);
        }
        if let Some(tx_hash) = &details.transaction_hash {
            let tx_url = format!("https://polygonscan.com/tx/{}", tx_hash);
            println!("{}TX:     {}{}{}", colors::MUTED, colors::ACCENT, tx_url, colors::RESET);
        }
        println!("{}{}{}", colors::HIGHLIGHT, "─".repeat(70), colors::RESET);
        println!();
        let _ = std::io::stdout().flush();

        let mut trade_log = format!("TRADE: {} - {}", Self::format_address(trader_address), action);
        if let Some(side) = &details.side {
            trade_log.push_str(&format!(" | Side: {}", side));
        }
        if let Some(amount) = details.amount {
            trade_log.push_str(&format!(" | Amount: ${:.2}", amount));
        }
        if let Some(price) = details.price {
            trade_log.push_str(&format!(" | Price: {}", price));
        }
        if let Some(title) = &details.title {
            trade_log.push_str(&format!(" | Market: {}", title));
        }
        if let Some(tx_hash) = &details.transaction_hash {
            trade_log.push_str(&format!(" | TX: {}", tx_hash));
        }
        Self::write_file(&trade_log);
    }

    pub fn balance(my_balance: f64, trader_balance: f64, trader_address: &str) {
        println!("{}Capital (USDC + Positions):{}", colors::MUTED, colors::RESET);
        println!(
            "{}  Your total capital:   {}$ {:.2}{}",
            colors::MUTED,
            format!("{}{}", colors::SUCCESS, colors::BOLD),
            my_balance,
            colors::RESET
        );
        println!(
            "{}  Trader total capital: {}$ {:.2} ({}){}",
            colors::MUTED,
            format!("{}{}", colors::ACCENT, colors::BOLD),
            trader_balance,
            Self::format_address(trader_address),
            colors::RESET
        );
        let _ = std::io::stdout().flush();
    }

    pub fn order_result(success: bool, message: &str) {
        if success {
            println!(
                "{} {} {}Order executed:{} {}",
                colors::SUCCESS,
                icons::OK,
                colors::SUCCESS,
                colors::RESET,
                message
            );
            let _ = std::io::stdout().flush();
            Self::write_file(&format!("ORDER SUCCESS: {}", message));
        } else {
            println!(
                "{} {} {}Order failed:{} {}",
                colors::ERROR,
                icons::ERR,
                colors::ERROR,
                colors::RESET,
                message
            );
            let _ = std::io::stdout().flush();
            Self::write_file(&format!("ORDER FAILED: {}", message));
        }
    }

    pub fn db_connection(traders: &[String], counts: &[u64]) {
        println!();
        println!("{}📦 Database Status:{}", colors::ACCENT, colors::RESET);
        for (i, addr) in traders.iter().enumerate() {
            let count = counts.get(i).copied().unwrap_or(0);
            println!(
                "{}   {}: {} {} trades{}",
                colors::MUTED,
                Self::format_address(addr),
                colors::WARN,
                count,
                colors::RESET
            );
        }
        println!();
        let _ = std::io::stdout().flush();
    }

    pub fn my_positions(
        wallet: &str,
        count: usize,
        top_positions: &[serde_json::Value],
        overall_pnl: f64,
        total_value: f64,
        initial_value: f64,
        current_balance: f64,
    ) {
        println!();
        println!(
            "{}💼 YOUR POSITIONS{}",
            format!("{}{}", colors::HIGHLIGHT, colors::BOLD),
            colors::RESET
        );
        println!("{}   Wallet: {}{}", colors::MUTED, Self::format_address(wallet), colors::RESET);
        println!();

        let total_portfolio = current_balance + total_value;
        println!(
            "{}   💰 Available Cash:    {}$ {:.2}{}",
            colors::MUTED,
            format!("{}{}", colors::WARN, colors::BOLD),
            current_balance,
            colors::RESET
        );
        println!(
            "{}   📊 Total Portfolio:   {}$ {:.2}{}",
            colors::MUTED,
            format!("{}{}", colors::ACCENT, colors::BOLD),
            total_portfolio,
            colors::RESET
        );

        if count == 0 {
            println!("{}   No open positions{}", colors::MUTED, colors::RESET);
        } else {
            println!();
            println!(
                "{}   📈 Open Positions:    {}{} position{}{}",
                colors::MUTED,
                colors::SUCCESS,
                count,
                if count > 1 { "s" } else { "" },
                colors::RESET
            );
            println!(
                "{}      Invested:          {}$ {:.2}{}",
                colors::MUTED,
                colors::MUTED,
                initial_value,
                colors::RESET
            );
            println!(
                "{}      Current Value:     {}$ {:.2}{}",
                colors::MUTED,
                colors::ACCENT,
                total_value,
                colors::RESET
            );
            let pnl_sign = if overall_pnl >= 0.0 { "+" } else { "" };
            let pnl_color = if overall_pnl >= 0.0 {
                colors::SUCCESS
            } else {
                colors::ERROR
            };
            println!(
                "{}      Profit/Loss:       {}{}{:.1}%{}",
                colors::MUTED,
                pnl_color,
                pnl_sign,
                overall_pnl,
                colors::RESET
            );

            if !top_positions.is_empty() {
                println!("{}   🔝 Top Positions:{}", colors::MUTED, colors::RESET);
                for pos in top_positions.iter().take(5) {
                    let percent_pnl = pos
                        .get("percentPnl")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let current_value = pos
                        .get("currentValue")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let avg_price = pos
                        .get("avgPrice")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let cur_price = pos
                        .get("curPrice")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let outcome = pos
                        .get("outcome")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown");
                    let title = pos
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown");
                    let title_display = if title.len() > 45 {
                        format!("{}...", &title[..45])
                    } else {
                        title.to_string()
                    };
                    let pnl_sign = if percent_pnl >= 0.0 { "+" } else { "" };
                    let pnl_color = if percent_pnl >= 0.0 {
                        colors::SUCCESS
                    } else {
                        colors::ERROR
                    };
                    println!(
                        "{}      • {} - {}{}",
                        colors::MUTED,
                        outcome,
                        title_display,
                        colors::RESET
                    );
                    println!(
                        "{}        Value: {}$ {:.2}{} | PnL: {}{}{:.1}%{}",
                        colors::MUTED,
                        colors::ACCENT,
                        current_value,
                        colors::RESET,
                        pnl_color,
                        pnl_sign,
                        percent_pnl,
                        colors::RESET
                    );
                    println!(
                        "{}        Bought @ {}{:.1}¢{} | Current @ {}{:.1}¢{}",
                        colors::MUTED,
                        colors::WARN,
                        avg_price * 100.0,
                        colors::RESET,
                        colors::WARN,
                        cur_price * 100.0,
                        colors::RESET
                    );
                }
            }
        }
        println!();
        let _ = std::io::stdout().flush();
    }

    pub fn traders_positions(
        traders: &[String],
        position_counts: &[usize],
        position_details: &[Vec<serde_json::Value>],
        profitabilities: &[f64],
    ) {
        println!("{}📈 TRADERS YOU'RE COPYING{}", colors::ACCENT, colors::RESET);
        for (i, addr) in traders.iter().enumerate() {
            let pos_count = position_counts.get(i).copied().unwrap_or(0);
            let pnl = profitabilities.get(i).copied().unwrap_or(0.0);
            let pnl_sign = if pnl >= 0.0 { "+" } else { "" };
            let pnl_color = if pnl >= 0.0 {
                colors::SUCCESS
            } else {
                colors::ERROR
            };
            println!(
                "{}   {}: {} positions · {}PnL {}{:.1}%{}",
                colors::MUTED,
                Self::format_address(addr),
                pos_count,
                pnl_color,
                pnl_sign,
                pnl,
                colors::RESET
            );

            if let Some(details) = position_details.get(i) {
                for pos in details.iter().take(3) {
                    let percent_pnl = pos
                        .get("percentPnl")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let current_value = pos
                        .get("currentValue")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let avg_price = pos
                        .get("avgPrice")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let cur_price = pos
                        .get("curPrice")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let outcome = pos
                        .get("outcome")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown");
                    let title = pos
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown");
                    let title_display = if title.len() > 40 {
                        format!("{}...", &title[..40])
                    } else {
                        title.to_string()
                    };
                    let pnl_sign = if percent_pnl >= 0.0 { "+" } else { "" };
                    let pnl_color = if percent_pnl >= 0.0 {
                        colors::SUCCESS
                    } else {
                        colors::ERROR
                    };
                    println!(
                        "{}     • {} - {}{}",
                        colors::MUTED,
                        outcome,
                        title_display,
                        colors::RESET
                    );
                    println!(
                        "{}       Value: {}$ {:.2}{} | PnL: {}{}{:.1}%{}",
                        colors::MUTED,
                        colors::ACCENT,
                        current_value,
                        colors::RESET,
                        pnl_color,
                        pnl_sign,
                        percent_pnl,
                        colors::RESET
                    );
                    println!(
                        "{}       Bought @ {}{:.1}¢{} | Current @ {}{:.1}¢{}",
                        colors::MUTED,
                        colors::WARN,
                        avg_price * 100.0,
                        colors::RESET,
                        colors::WARN,
                        cur_price * 100.0,
                        colors::RESET
                    );
                }
            }
        }
        println!();
        let _ = std::io::stdout().flush();
    }
}

#[derive(Clone)]
pub struct TradeDetails {
    pub asset: Option<String>,
    pub side: Option<String>,
    pub amount: Option<f64>,
    pub price: Option<f64>,
    pub slug: Option<String>,
    pub event_slug: Option<String>,
    pub transaction_hash: Option<String>,
    pub title: Option<String>,
}
