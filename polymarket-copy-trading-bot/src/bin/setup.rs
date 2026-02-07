use polymarket_copy_rust::utils::theme::colors;
use std::fs;
use std::path::Path;

fn main() {
    let example = Path::new(".env.example");
    let env = Path::new(".env");

    println!();
    println!(
        "{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
        colors::ACCENT
    );
    println!("     POLYMARKET COPY TRADING BOT — SETUP");
    println!(
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}",
        colors::RESET
    );
    println!();

    if env.exists() {
        println!(
            "{} .env already exists. Edit it to change configuration.{}",
            colors::SUCCESS,
            colors::RESET
        );
        println!();
        println!("  Validate: make validate-setup");
        println!("  Health:   make health-check");
        println!();
        return;
    }

    if !example.exists() {
        eprintln!(
            "{} .env.example not found. Cannot create .env.{}",
            colors::ERROR,
            colors::RESET
        );
        std::process::exit(1);
    }

    match fs::copy(example, env) {
        Ok(_) => {
            println!(
                "{} Created .env from .env.example.{}",
                colors::SUCCESS,
                colors::RESET
            );
            println!();
            println!("  Next steps:");
            println!("  1. Open .env and set: USER_ADDRESSES, PROXY_WALLET, PRIVATE_KEY,");
            println!("     MONGO_URI, RPC_URL, CLOB_HTTP_URL, CLOB_WS_URL, USDC_CONTRACT_ADDRESS");
            println!("  2. Run: make validate-setup");
            println!("  3. Run: make health-check");
            println!("  4. Run: make run");
            println!();
        }
        Err(e) => {
            eprintln!(
                "{} Failed to create .env: {}{}",
                colors::ERROR,
                e,
                colors::RESET
            );
            std::process::exit(1);
        }
    }
}
