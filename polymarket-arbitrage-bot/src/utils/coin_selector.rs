use crate::config::AVAILABLE_COINS;
use colored::*;

pub fn display_coin_selection(selected_index: usize) {
    print!("\x1B[2J\x1B[1;1H"); // Clear screen
    println!("{}", "\n╔════════════════════════════════════════════════════════════════╗".cyan().bold());
    println!("{}", "║     Polymarket Arbitrage Bot - Coin Selection                  ║".cyan().bold());
    println!("{}", "╚════════════════════════════════════════════════════════════════╝\n".cyan().bold());
    
    println!("{}", "Select a coin to monitor:\n".yellow());
    println!("{}", "Use ↑/↓ arrow keys to navigate, Enter to select\n".bright_black());
    
    for (index, coin) in AVAILABLE_COINS.iter().enumerate() {
        let is_selected = index == selected_index;
        let prefix = if is_selected { "> " } else { "  " };
        let coin_color = if is_selected {
            coin.cyan().bold()
        } else {
            coin.white()
        };
        
        if is_selected {
            println!("{}{}", prefix.on_blue(), coin_color);
        } else {
            println!("{}{}", prefix, coin_color);
        }
    }
    
    println!("{}", format!("\n{}", "─".repeat(60)).bright_black());
}

pub fn get_available_coins() -> Vec<&'static str> {
    AVAILABLE_COINS.to_vec()
}

