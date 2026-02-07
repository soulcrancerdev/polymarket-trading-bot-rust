use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::{BotCommand, InlineKeyboardButton, InlineKeyboardMarkup, MessageId};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Command as TokioCommand, Child as TokioChild};
use tokio::sync::Mutex;
use std::process::Stdio;

type ProcessMap = Arc<Mutex<HashMap<String, (TokioChild, MessageId)>>>;

fn get_user_config_path(user_id: i64) -> PathBuf {
    PathBuf::from("users").join(format!("{}", user_id))
}

fn ensure_users_directory() -> std::io::Result<()> {
    fs::create_dir_all("users")
}

fn initialize_user_config(user_id: i64) -> std::io::Result<()> {
    ensure_users_directory()?;
    let user_config_path = get_user_config_path(user_id);
    
    if user_config_path.exists() {
        return Ok(());
    }
    
    let template_content = if Path::new(".config.example").exists() {
        let content = fs::read_to_string(".config.example")?;
        process_template_content(&content)
    } else {
        "# User Configuration File\n# Generated automatically\n\n".to_string()
    };
    
    fs::write(&user_config_path, template_content)?;
    Ok(())
}

fn process_template_content(content: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    let default_vars: Vec<(&str, &str)> = vec![
        ("COPY_STRATEGY", "PERCENTAGE"),
        ("COPY_SIZE", "10.0"),
        ("MAX_ORDER_SIZE_USD", "100.0"),
        ("MIN_ORDER_SIZE_USD", "1.0"),
    ];
    
    let mut found_vars: std::collections::HashSet<&str> = std::collections::HashSet::new();
    
    for line in content.lines() {
        let trimmed = line.trim();
        let mut processed = false;
        
        for (var, default_value) in &default_vars {
            if trimmed.contains(&format!("{}=", var)) {
                found_vars.insert(*var);
                
                if trimmed.starts_with("#") {
                    let uncommented = trimmed.trim_start_matches('#').trim_start();
                    if let Some(equals_pos) = uncommented.find('=') {
                        let var_part = &uncommented[..equals_pos].trim();
                        let value_part = &uncommented[equals_pos + 1..].trim();
                        if value_part.is_empty() || value_part.contains("your_") || value_part.contains("here") {
                            lines.push(format!("{}={}", var, default_value));
                        } else {
                            lines.push(uncommented.to_string());
                        }
                    } else {
                        lines.push(uncommented.to_string());
                    }
                } else {
                    lines.push(line.to_string());
                }
                processed = true;
                break;
            }
        }
        
        if !processed {
            lines.push(line.to_string());
        }
    }
    
    let mut result = lines.join("\n");
    for (var, default_value) in &default_vars {
        if !found_vars.contains(*var) {
            if let Some(pos) = result.find("# OPTIONAL SETTINGS") {
                if let Some(newline_pos) = result[pos..].find('\n') {
                    let insert_pos = pos + newline_pos + 1;
                    result.insert_str(insert_pos, &format!("\n{}={}", var, default_value));
                }
            } else {
                result.push_str(&format!("\n{}={}", var, default_value));
            }
        }
    }
    
    result
}

fn read_user_config(user_id: i64) -> String {
    let user_config_path = get_user_config_path(user_id);
    fs::read_to_string(&user_config_path).unwrap_or_default()
}

fn write_user_config(user_id: i64, content: &str) -> std::io::Result<()> {
    let user_config_path = get_user_config_path(user_id);
    fs::write(&user_config_path, content)
}

fn parse_env_file(content: &str) -> HashMap<String, String> {
    content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                Some((parts[0].trim().to_string(), parts[1].trim().to_string()))
            } else {
                None
            }
        })
        .collect()
}

fn format_env_file(vars: &HashMap<String, String>) -> String {
    let mut lines = Vec::new();
    for (key, value) in vars.iter() {
        lines.push(format!("{}={}", key, value));
    }
    lines.join("\n")
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    
    let bot_token = std::env::var("TELEGRAM_BOT_TOKEN")
        .expect("TELEGRAM_BOT_TOKEN environment variable is required. Get it from @BotFather on Telegram.");
    
    let bot = Bot::new(bot_token);
    let processes: ProcessMap = Arc::new(Mutex::new(HashMap::new()));
    
    println!("🤖 Telegram bot starting...");
    
    let commands = vec![
        BotCommand::new("start", "get started with the bot"),
        BotCommand::new("menu", "view main menu"),
        BotCommand::new("help", "tips and frequently asked questions"),
    ];
    if let Err(e) = bot.set_my_commands(commands).await {
        eprintln!("⚠️ Failed to set bot commands: {}", e);
    } else {
        println!("✅ Bot commands menu set successfully");
    }
    
    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(handle_message_command))
        .branch(Update::filter_callback_query().endpoint(handle_callback_query));
    
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![processes])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

async fn handle_message_command(bot: Bot, msg: Message) -> ResponseResult<()> {
    if let Some(text) = msg.text() {
        match text {
            "/start" | "/menu" => {
                let user_id = msg.chat.id.0;
                let username = msg.chat.username().map(|s| s.to_string()).unwrap_or_else(|| "N/A".to_string());
                let first_name = msg.chat.first_name().map(|s| s.to_string()).unwrap_or_else(|| "N/A".to_string());
                
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("👤 User Started Bot");
                println!("   User ID: {}", user_id);
                println!("   Username: @{}", username);
                println!("   Name: {}", first_name);
                println!("   Config: users/{}", user_id);
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                
                if let Err(e) = initialize_user_config(user_id) {
                    eprintln!("⚠️ Failed to initialize user config for {}: {}", user_id, e);
                } else {
                    println!("✅ User config file initialized: users/{}", user_id);
                }
                
                send_main_menu(&bot, msg.chat.id).await?;
            }
            "/help" => {
                handle_help_command(&bot, msg.chat.id).await?;
            }
            cmd if cmd.starts_with("/set ") => {
                handle_set_command(&bot, msg.chat.id, cmd, Some(msg.id)).await?;
            }
            _ => {
                bot.send_message(msg.chat.id, "Use /start or /menu to see the main menu.")
                    .await?;
            }
        }
    }
    Ok(())
}

async fn handle_callback_query(
    bot: Bot,
    q: CallbackQuery,
    processes: ProcessMap,
) -> ResponseResult<()> {
    if let Some(data) = q.data {
        if let Some(msg) = q.message {
            let chat_id = msg.chat.id;
            
            match data.as_str() {
                "manage_env" => {
                    handle_manage_env_with_edit(&bot, chat_id, Some(msg.id)).await?;
                }
                "validate_setup" => {
                    handle_validate_setup(&bot, chat_id, processes.clone(), Some(msg.id)).await?;
                }
                "run_setup" => {
                    handle_run_binary(&bot, chat_id, "setup", processes.clone(), msg.id).await?;
                }
                "run_health_check" => {
                    handle_run_binary(&bot, chat_id, "health_check", processes.clone(), msg.id).await?;
                }
                "run_check_allowance" => {
                    handle_run_binary(&bot, chat_id, "check_allowance", processes.clone(), msg.id).await?;
                }
                "run_verify_allowance" => {
                    handle_run_binary(&bot, chat_id, "verify_allowance", processes.clone(), msg.id).await?;
                }
                "run_set_token_allowance" => {
                    handle_run_binary(&bot, chat_id, "set_token_allowance", processes.clone(), msg.id).await?;
                }
                "run_check_proxy" => {
                    handle_run_binary(&bot, chat_id, "check_proxy", processes.clone(), msg.id).await?;
                }
                "run_check_both" => {
                    handle_run_binary(&bot, chat_id, "check_both", processes.clone(), msg.id).await?;
                }
                "run_check_stats" => {
                    handle_run_binary(&bot, chat_id, "check_stats", processes.clone(), msg.id).await?;
                }
                "run_check_activity" => {
                    handle_run_binary(&bot, chat_id, "check_activity", processes.clone(), msg.id).await?;
                }
                "run_check_pnl" => {
                    handle_run_binary(&bot, chat_id, "check_pnl", processes.clone(), msg.id).await?;
                }
                "run_manual_sell" => {
                    handle_run_binary(&bot, chat_id, "manual_sell", processes.clone(), msg.id).await?;
                }
                "run_sell_large" => {
                    handle_run_binary(&bot, chat_id, "sell_large", processes.clone(), msg.id).await?;
                }
                "run_close_stale" => {
                    handle_run_binary(&bot, chat_id, "close_stale", processes.clone(), msg.id).await?;
                }
                "run_close_resolved" => {
                    handle_run_binary(&bot, chat_id, "close_resolved", processes.clone(), msg.id).await?;
                }
                "run_redeem_resolved" => {
                    handle_run_binary(&bot, chat_id, "redeem_resolved", processes.clone(), msg.id).await?;
                }
                "run_transfer_to_gnosis" => {
                    handle_run_binary(&bot, chat_id, "transfer_to_gnosis", processes.clone(), msg.id).await?;
                }
                "run_find_traders" => {
                    handle_run_binary(&bot, chat_id, "find_traders", processes.clone(), msg.id).await?;
                }
                "run_find_low_risk" => {
                    handle_run_binary(&bot, chat_id, "find_low_risk", processes.clone(), msg.id).await?;
                }
                "run_scan_traders" => {
                    handle_run_binary(&bot, chat_id, "scan_traders", processes.clone(), msg.id).await?;
                }
                "run_scan_markets" => {
                    handle_run_binary(&bot, chat_id, "scan_markets", processes.clone(), msg.id).await?;
                }
                "run_simulate" => {
                    handle_run_binary(&bot, chat_id, "simulate", processes.clone(), msg.id).await?;
                }
                "run_simulate_old" => {
                    handle_run_binary(&bot, chat_id, "simulate_old", processes.clone(), msg.id).await?;
                }
                "run_sim" => {
                    handle_run_binary(&bot, chat_id, "sim", processes.clone(), msg.id).await?;
                }
                "run_compare" => {
                    handle_run_binary(&bot, chat_id, "compare", processes.clone(), msg.id).await?;
                }
                "run_fetch_history" => {
                    handle_run_binary(&bot, chat_id, "fetch_history", processes.clone(), msg.id).await?;
                }
                "run_aggregate" => {
                    handle_run_binary(&bot, chat_id, "aggregate", processes.clone(), msg.id).await?;
                }
                "run_audit" => {
                    handle_run_binary(&bot, chat_id, "audit", processes.clone(), msg.id).await?;
                }
                "run_audit_old" => {
                    handle_run_binary(&bot, chat_id, "audit_old", processes.clone(), msg.id).await?;
                }
                "run_main_bot" => {
                    handle_run_binary(&bot, chat_id, "polymarket-copy-rust", processes.clone(), msg.id).await?;
                }
                "more_commands" => {
                    bot.answer_callback_query(q.id.clone())
                        .text("This is a premium feature")
                        .show_alert(true)
                        .await?;
                    return Ok(());
                }
                "back_to_menu" => {
                    send_main_menu_with_edit(&bot, chat_id, Some(msg.id)).await?;
                }
                cmd if cmd.starts_with("stop_") => {
                    let binary_name = cmd.strip_prefix("stop_").unwrap();
                    handle_stop_process(&bot, chat_id, binary_name, processes.clone()).await?;
                }
                cmd if cmd.starts_with("set_env_") => {
                    let var_name = cmd.strip_prefix("set_env_").unwrap();
                    if var_name == "TRADE_AGGREGATION_ENABLED" {
                        handle_prompt_bool_var(&bot, chat_id, var_name, msg.id).await?;
                    } else {
                        handle_prompt_env_var(&bot, chat_id, var_name, msg.id).await?;
                    }
                }
                cmd if cmd.starts_with("set_bool_") => {
                    let rest = cmd.strip_prefix("set_bool_").unwrap();
                    if let Some((var_name, value)) = rest.rsplit_once('_') {
                        let bool_value = value == "true";
                        handle_set_bool_var(&bot, chat_id, var_name, bool_value, msg.id).await?;
                    }
                }
                _ => {}
            }
        } else {
            return Ok(());
        }
        
        bot.answer_callback_query(q.id).await?;
    }
    Ok(())
}

async fn send_main_menu(bot: &Bot, chat_id: ChatId) -> ResponseResult<()> {
    send_main_menu_with_edit(bot, chat_id, None).await
}

async fn send_main_menu_with_edit(bot: &Bot, chat_id: ChatId, edit_msg_id: Option<MessageId>) -> ResponseResult<()> {
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("⚙️ Manage Environment Variables", "manage_env")],
        vec![InlineKeyboardButton::callback("🏥 Health Check", "run_health_check")],
        vec![
            InlineKeyboardButton::callback("💰 Check Allowance", "run_check_allowance"),
            InlineKeyboardButton::callback("💵 Check PnL", "run_check_pnl")
        ],
        vec![InlineKeyboardButton::callback("▶️ Run Main Bot", "run_main_bot")],
        vec![InlineKeyboardButton::callback("📋 More Commands(premium)", "more_commands")],
    ]);
    
    let message_text = "🤖 *Polymarket Copy Trading Bot*\n\nSelect an action:";
    
    if let Some(msg_id) = edit_msg_id {
        bot.edit_message_text(chat_id, msg_id, message_text)
            .reply_markup(keyboard)
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?;
    } else {
        bot.send_message(chat_id, message_text)
            .reply_markup(keyboard)
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?;
    }
    
    Ok(())
}

async fn handle_help_command(bot: &Bot, chat_id: ChatId) -> ResponseResult<()> {
    let help_text = r#"📖 *Help & FAQ*

*Getting Started:*
1\. Use `/start` or `/menu` to open the main menu
2\. Click "⚙️ Manage Environment Variables" to configure your settings
3\. Click "✅ Validate Setup" to verify your configuration
4\. Use `/set VAR_NAME value` to set environment variables

*Main Commands:*
• `/start` or `/menu` \- Open main menu
• `/help` \- Show this help message
• `/set VAR_NAME value` \- Set an environment variable

*Environment Variables:*
• `USER_ADDRESSES` \- Comma\-separated trader addresses to copy
• `PROXY_WALLET` \- Your wallet address for executing trades
• `PRIVATE_KEY` \- Your wallet's private key \(keep secret\!\)
• `CLOB_HTTP_URL` \- Polymarket CLOB HTTP endpoint
• `CLOB_WS_URL` \- Polymarket CLOB WebSocket endpoint
• `RPC_URL` \- Blockchain RPC URL
• `USDC_CONTRACT_ADDRESS` \- USDC contract address

*Tips:*
• Boolean variables \(like `TRADE_AGGREGATION_ENABLED`\) show True/False buttons
• Other variables use `/set VAR_NAME value` command
• Your config is stored in `users/{your_user_id}` file
• Always validate your setup before running the bot

*Need more help?*
Check the README\.md file or contact support\."#;

    bot.send_message(chat_id, help_text)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await?;
    
    Ok(())
}

async fn handle_more_commands(bot: &Bot, chat_id: ChatId) -> ResponseResult<()> {
    handle_more_commands_with_edit(bot, chat_id, None).await
}

async fn handle_more_commands_with_edit(bot: &Bot, chat_id: ChatId, edit_msg_id: Option<MessageId>) -> ResponseResult<()> {
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("🔐 Verify Allowance", "run_verify_allowance"),
            InlineKeyboardButton::callback("🔑 Set Token Allowance", "run_set_token_allowance")
        ],
        vec![
            InlineKeyboardButton::callback("🔍 Check Proxy", "run_check_proxy"),
            InlineKeyboardButton::callback("🔍 Check Both", "run_check_both")
        ],
        vec![
            InlineKeyboardButton::callback("💸 Manual Sell", "run_manual_sell"),
            InlineKeyboardButton::callback("💸 Sell Large", "run_sell_large")
        ],
        vec![
            InlineKeyboardButton::callback("⏰ Close Stale", "run_close_stale"),
            InlineKeyboardButton::callback("✅ Close Resolved", "run_close_resolved")
        ],
        vec![
            InlineKeyboardButton::callback("🎁 Redeem Resolved", "run_redeem_resolved"),
            InlineKeyboardButton::callback("🔐 Transfer to Gnosis", "run_transfer_to_gnosis")
        ],
        vec![
            InlineKeyboardButton::callback("🔍 Find Traders", "run_find_traders"),
            InlineKeyboardButton::callback("🛡️ Find Low Risk", "run_find_low_risk")
        ],
        vec![
            InlineKeyboardButton::callback("🔍 Scan Traders", "run_scan_traders"),
            InlineKeyboardButton::callback("📊 Scan Markets", "run_scan_markets")
        ],
        vec![
            InlineKeyboardButton::callback("🧪 Simulate", "run_simulate"),
            InlineKeyboardButton::callback("🧪 Simulate Old", "run_simulate_old")
        ],
        vec![
            InlineKeyboardButton::callback("🧪 Sim", "run_sim"),
            InlineKeyboardButton::callback("📊 Compare", "run_compare")
        ],
        vec![
            InlineKeyboardButton::callback("📜 Fetch History", "run_fetch_history"),
            InlineKeyboardButton::callback("📊 Aggregate", "run_aggregate")
        ],
        vec![
            InlineKeyboardButton::callback("🔍 Audit", "run_audit"),
            InlineKeyboardButton::callback("🔍 Audit Old", "run_audit_old")
        ],
        vec![InlineKeyboardButton::callback("◀️ Back to Menu", "back_to_menu")],
    ]);
    
    let message_text = "📋 *More Commands*\n\nSelect a command:";
    
    if let Some(msg_id) = edit_msg_id {
        bot.edit_message_text(chat_id, msg_id, message_text)
            .reply_markup(keyboard)
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?;
    } else {
        bot.send_message(chat_id, message_text)
            .reply_markup(keyboard)
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?;
    }
    
    Ok(())
}

async fn handle_manage_env(bot: &Bot, chat_id: ChatId) -> ResponseResult<()> {
    handle_manage_env_with_edit(bot, chat_id, None).await
}

async fn handle_manage_env_with_edit(bot: &Bot, chat_id: ChatId, edit_msg_id: Option<MessageId>) -> ResponseResult<()> {
    let common_vars = vec![
        "USER_ADDRESSES",
        "PROXY_WALLET",
        "PRIVATE_KEY",
        "RPC_URL",
        "MONGO_URI",
        "COPY_STRATEGY",
        "COPY_SIZE",
        "MAX_ORDER_SIZE_USD",
        "MIN_ORDER_SIZE_USD",
    ];
    
    let user_id = chat_id.0;
    let current_env = read_user_config(user_id);
    let current_vars = parse_env_file(&current_env);
    
    let mut keyboard_buttons: Vec<Vec<InlineKeyboardButton>> = Vec::new();
    
    for var in common_vars.iter() {
        let value = current_vars.get(*var).cloned().unwrap_or_default();
        let display = if value.is_empty() {
            format!("{}: (not set)", var)
        } else if var.contains("KEY") || var.contains("PRIVATE") || var.contains("ADDRESS") {
            format!("{}: ***", var)
        } else {
            let truncated = if value.len() > 20 {
                format!("{}...", &value[..20])
            } else {
                value
            };
            format!("{}: {}", var, truncated)
        };
        keyboard_buttons.push(vec![
            InlineKeyboardButton::callback(display, format!("set_env_{}", var))
        ]);
    }
    
    keyboard_buttons.push(vec![InlineKeyboardButton::callback("◀️ Back to Menu", "back_to_menu")]);
    
    let keyboard = InlineKeyboardMarkup::new(keyboard_buttons);
    let message_text = "⚙️ *Environment Variables*\n\nSelect a variable to edit:\n\nUse `/set VAR\\_NAME value` to set a variable\\.";
    
    if let Some(msg_id) = edit_msg_id {
        bot.edit_message_text(chat_id, msg_id, message_text)
            .reply_markup(keyboard)
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?;
    } else {
        bot.send_message(chat_id, message_text)
            .reply_markup(keyboard)
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?;
    }
    
    Ok(())
}

async fn handle_prompt_env_var(bot: &Bot, chat_id: ChatId, var_name: &str, callback_msg_id: MessageId) -> ResponseResult<()> {
    let message = match var_name {
        "USER_ADDRESSES" => {
            format!(
                "📝 To set `{}`, use:\n\n`/set {} your\\_value\\_here`\n\n*Comma\\-separated list of trader addresses to copy from*\n\n*Multiple addresses can be set in two ways:*\n\n1\\. *Comma\\-separated:*\n`/set {} 0x1234...,0x5678...,0x9abc...`\n\n2\\. *JSON array:*\n`/set {} [\"0x1234...\", \"0x5678...\", \"0x9abc...\"]`\n\n*Example:*\n`/set {} 0x1234567890123456789012345678901234567890,0xabcdefabcdefabcdefabcdefabcdefabcdefabcd`",
                var_name, var_name, var_name, var_name, var_name
            )
        }
        "PROXY_WALLET" => {
            format!(
                "📝 To set `{}`, use:\n\n`/set {} your\\_value\\_here`\n\n*Your proxy wallet address \\(the wallet that will execute trades\\)*\n\n*Example:*\n`/set {} 0x1234567890123456789012345678901234567890`",
                var_name, var_name, var_name
            )
        }
        "PRIVATE_KEY" => {
            format!(
                "📝 To set `{}`, use:\n\n`/set {} your\\_value\\_here`\n\n*Your wallet's private key \\(64\\-character hex string, no 0x prefix\\)*\n\n⚠️ *KEEP THIS SECRET\\! Never share or commit to git\\!*\n\n*Example:*\n`/set {} 0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef`",
                var_name, var_name, var_name
            )
        }
        "RPC_URL" => {
            format!(
                "📝 To set `{}`, use:\n\n`/set {} your\\_value\\_here`\n\n*RPC URL for blockchain interactions*\n\n*Example:*\n`/set {} https://polygon\\-rpc\\.com`",
                var_name, var_name, var_name
            )
        }
        "MONGO_URI" => {
            format!(
                "📝 To set `{}`, use:\n\n`/set {} your\\_value\\_here`\n\n*MongoDB connection URI \\(optional, defaults to localhost\\)*\n\n*Example:*\n`/set {} mongodb://localhost:27017/polymarket\\_copytrading`",
                var_name, var_name, var_name
            )
        }
        "COPY_STRATEGY" => {
            format!(
                "📝 To set `{}`, use:\n\n`/set {} your\\_value\\_here`\n\n*Copy strategy: PERCENTAGE, FIXED, or ADAPTIVE*\n\n*Example:*\n`/set {} PERCENTAGE`",
                var_name, var_name, var_name
            )
        }
        "COPY_SIZE" => {
            format!(
                "📝 To set `{}`, use:\n\n`/set {} your\\_value\\_here`\n\n*Copy size \\(percentage for PERCENTAGE strategy, USD for FIXED\\)*\n\n*Example:*\n`/set {} 10\\.0`",
                var_name, var_name, var_name
            )
        }
        "MAX_ORDER_SIZE_USD" => {
            format!(
                "📝 To set `{}`, use:\n\n`/set {} your\\_value\\_here`\n\n*Maximum order size in USD*\n\n*Example:*\n`/set {} 100\\.0`",
                var_name, var_name, var_name
            )
        }
        "MIN_ORDER_SIZE_USD" => {
            format!(
                "📝 To set `{}`, use:\n\n`/set {} your\\_value\\_here`\n\n*Minimum order size in USD*\n\n*Example:*\n`/set {} 1\\.0`",
                var_name, var_name, var_name
            )
        }
        _ => {
            format!("📝 To set `{}`, use:\n\n`/set {} your\\_value\\_here`\n\n*Example:*\n`/set {} abc123`", var_name, var_name, var_name)
        }
    };
    
    let keyboard = InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("◀️ Back to Menu", "manage_env")
    ]]);
    
    bot.edit_message_text(chat_id, callback_msg_id, message)
        .reply_markup(keyboard)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await?;
    
    Ok(())
}

async fn handle_prompt_bool_var(bot: &Bot, chat_id: ChatId, var_name: &str, callback_msg_id: MessageId) -> ResponseResult<()> {
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("✅ True", format!("set_bool_{}_true", var_name)),
            InlineKeyboardButton::callback("❌ False", format!("set_bool_{}_false", var_name)),
        ],
        vec![InlineKeyboardButton::callback("◀️ Back to Menu", "manage_env")],
    ]);
    
    bot.edit_message_text(chat_id, callback_msg_id, format!("📝 Select value for `{}`:", var_name))
        .reply_markup(keyboard)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await?;
    
    Ok(())
}

async fn handle_set_bool_var(bot: &Bot, chat_id: ChatId, var_name: &str, value: bool, callback_msg_id: MessageId) -> ResponseResult<()> {
    let user_id = chat_id.0;
    let value_str = if value { "true" } else { "false" };
    
    if let Err(e) = initialize_user_config(user_id) {
        bot.send_message(chat_id, format!("❌ Failed to access your config file: {}", e))
            .await?;
        return Ok(());
    }
    
    let env_content = read_user_config(user_id);
    let mut lines: Vec<String> = env_content.lines().map(|s| s.to_string()).collect();
    
    let mut found = false;
    for line in lines.iter_mut() {
        if line.trim().starts_with(&format!("{}=", var_name)) {
            *line = format!("{}={}", var_name, value_str);
            found = true;
            break;
        }
    }
    
    if !found {
        lines.push(format!("{}={}", var_name, value_str));
    }
    
    write_user_config(user_id, &lines.join("\n"))
        .map_err(|e| teloxide::RequestError::Io(e))?;
    
    handle_manage_env_with_edit(bot, chat_id, Some(callback_msg_id)).await?;
    
    Ok(())
}

async fn handle_set_command(bot: &Bot, chat_id: ChatId, cmd: &str, msg_id: Option<MessageId>) -> ResponseResult<()> {
    let parts: Vec<&str> = cmd.splitn(3, ' ').collect();
    if parts.len() < 3 {
        bot.send_message(chat_id, "❌ Invalid format. Use: `/set VAR_NAME value`")
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?;
        return Ok(());
    }
    
    let var_name = parts[1];
    let value = parts[2];
    let user_id = chat_id.0;
    
    if let Err(e) = initialize_user_config(user_id) {
        bot.send_message(chat_id, format!("❌ Failed to access your config file: {}", e))
            .await?;
        return Ok(());
    }
    
    let env_content = read_user_config(user_id);
    let mut lines: Vec<String> = env_content.lines().map(|s| s.to_string()).collect();
    
    let mut found = false;
    for line in lines.iter_mut() {
        if line.trim().starts_with(&format!("{}=", var_name)) {
            *line = format!("{}={}", var_name, value);
            found = true;
            break;
        }
    }
    
    if !found {
        lines.push(format!("{}={}", var_name, value));
    }
    
    match write_user_config(user_id, &lines.join("\n")) {
        Ok(_) => {
            if let Some(cmd_msg_id) = msg_id {
                let _ = bot.delete_message(chat_id, cmd_msg_id).await;
            }
            
            if let Some(cmd_msg_id) = msg_id {
                if cmd_msg_id.0 > 1 {
                    let _ = bot.delete_message(chat_id, MessageId(cmd_msg_id.0 - 1)).await;
                }
            }
            
            handle_manage_env(bot, chat_id).await?;
        }
        Err(e) => {
            bot.send_message(chat_id, format!("❌ Failed to write config file: {}", e))
                .await?;
        }
    }
    
    Ok(())
}

async fn handle_validate_setup(
    bot: &Bot,
    chat_id: ChatId,
    processes: ProcessMap,
    edit_msg_id: Option<MessageId>,
) -> ResponseResult<()> {
    let procs: tokio::sync::MutexGuard<'_, HashMap<String, (TokioChild, MessageId)>> = processes.lock().await;
    if procs.contains_key("validate_setup") {
        bot.send_message(chat_id, "⚠️ Validate setup is already running!")
            .await?;
        return Ok(());
    }
    drop(procs);
    
    let user_id = chat_id.0;
    
    if let Err(e) = initialize_user_config(user_id) {
        bot.send_message(chat_id, format!("❌ Failed to initialize your config file: {}", e))
            .await?;
        return Ok(());
    }
    
    let status_msg = if let Some(msg_id) = edit_msg_id {
        bot.edit_message_text(chat_id, msg_id, format!("🔄 Validating your configuration (users/{})...", user_id))
            .await?
    } else {
        bot.send_message(chat_id, format!("🔄 Validating your configuration (users/{})...", user_id))
            .await?
    };
    
    let binary_path = std::path::PathBuf::from("target/release/validate_setup");
    let mut cmd = TokioCommand::new(&binary_path);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.current_dir(".");
    
    let user_config_content = read_user_config(user_id);
    let env_vars = parse_env_file(&user_config_content);
    for (key, value) in env_vars.iter() {
        cmd.env(key, value);
    }
    
    let child = cmd.spawn()
        .map_err(|e| teloxide::RequestError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to spawn process: {}", e))))?;
    
    let msg_id = status_msg.id;
    {
        let mut procs: tokio::sync::MutexGuard<'_, HashMap<String, (TokioChild, MessageId)>> = processes.lock().await;
        procs.insert("validate_setup".to_string(), (child, msg_id));
    }
    
    let bot_clone = bot.clone();
    let chat_id_clone = chat_id;
    let processes_clone = processes.clone();
    
    tokio::spawn(async move {
        let mut full_log = String::new();
        let mut update_buffer = String::new();
        
        let mut child_opt = None;
        {
            let mut procs = processes_clone.lock().await;
            if let Some((child, _)) = procs.remove("validate_setup") {
                child_opt = Some(child);
            }
        }
        
        if let Some(mut child) = child_opt {
            if let Some(stdout) = child.stdout.take() {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                
                while let Ok(Some(line)) = lines.next_line().await {
                    full_log.push_str(&line);
                    full_log.push('\n');
                    update_buffer.push_str(&line);
                    update_buffer.push('\n');
                    
                    if update_buffer.lines().count() >= 10 || update_buffer.len() > 1500 {
                        let formatted_log = format_output_with_emoji(&full_log, Some("validate_setup"));
                        let _ = bot_clone.edit_message_text(
                            chat_id_clone,
                            msg_id,
                            format!("🔄 *Validate Setup*\n```\n{}\n```", escape_markdown(&formatted_log)),
                        )
                        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                        .await;
                        update_buffer.clear();
                    }
                }
            }
            
            let _ = child.wait().await;
        }
        
        let keyboard = InlineKeyboardMarkup::new(vec![vec![
            InlineKeyboardButton::callback("◀️ Back to Menu", "back_to_menu")
        ]]);
        
        let final_message = if full_log.trim().is_empty() {
            "✅ *Validate Setup Completed\\!*".to_string()
        } else {
            let formatted_log = format_output_with_emoji(&full_log, Some("validate_setup"));
            format!("🔄 *Validate Setup*\n```\n{}\n```\n\n✅ *Validation Completed\\!*", escape_markdown(&formatted_log))
        };
        
        let _ = bot_clone.edit_message_text(chat_id_clone, msg_id, final_message)
            .reply_markup(keyboard)
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await;
    });
    
    Ok(())
}

async fn handle_run_binary(
    bot: &Bot,
    chat_id: ChatId,
    binary_name: &str,
    processes: ProcessMap,
    callback_msg_id: MessageId,
) -> ResponseResult<()> {
    let procs: tokio::sync::MutexGuard<'_, HashMap<String, (TokioChild, MessageId)>> = processes.lock().await;
    if procs.contains_key(binary_name) {
        bot.edit_message_text(chat_id, callback_msg_id, format!("⚠️ {} is already running!", binary_name))
            .await?;
        return Ok(());
    }
    drop(procs);
    
    let user_id = chat_id.0;
    
    if let Err(e) = initialize_user_config(user_id) {
        bot.edit_message_text(chat_id, callback_msg_id, format!("❌ Failed to initialize your config file: {}", e))
            .await?;
        return Ok(());
    }
    
    let keyboard = InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("🛑 Stop", format!("stop_{}", binary_name))
    ]]);
    
    let status_msg = bot.edit_message_text(
        chat_id,
        callback_msg_id,
        format!("🔄 *{}* is running\\.\\.\\.\n\n```\n\n```", escape_markdown(binary_name)),
    )
    .reply_markup(keyboard)
    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
    .await?;
    
    let binary_path = std::path::PathBuf::from(format!("target/release/{}", binary_name));
    let mut cmd = TokioCommand::new(&binary_path);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.current_dir(".");
    
    let user_config_content = read_user_config(user_id);
    let env_vars = parse_env_file(&user_config_content);
    for (key, value) in env_vars.iter() {
        cmd.env(key, value);
    }
    
    let child = cmd.spawn()
        .map_err(|e| teloxide::RequestError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to spawn process: {}", e))))?;
    
    let msg_id = status_msg.id;
    let binary_name_str = binary_name.to_string();
    {
        let mut procs: tokio::sync::MutexGuard<'_, HashMap<String, (TokioChild, MessageId)>> = processes.lock().await;
        procs.insert(binary_name_str.clone(), (child, msg_id));
    }
    
    let bot_clone = bot.clone();
    let chat_id_clone = chat_id;
    let processes_clone = processes.clone();
    
    tokio::spawn(async move {
        let mut full_log = String::new();
        
        let mut stdout_opt = None;
        let mut stderr_opt = None;
        {
            let mut procs = processes_clone.lock().await;
            if let Some((child, _)) = procs.get_mut(&binary_name_str) {
                stdout_opt = child.stdout.take();
                stderr_opt = child.stderr.take();
            }
        }
        
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        
        if let Some(stdout) = stdout_opt {
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let _ = tx_clone.send(line);
                }
            });
        }
        
        if let Some(stderr) = stderr_opt {
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let _ = tx_clone.send(line);
                }
            });
        }
        
        drop(tx);
        
        let truncate_log = |log: &str| -> String {
            const MAX_CHARS: usize = 3500;
            if log.len() <= MAX_CHARS {
                return log.to_string();
            }
            
            let lines: Vec<&str> = log.lines().collect();
            let mut result = String::with_capacity(MAX_CHARS);
            let truncate_msg = "\n... (showing recent logs only, older logs truncated) ...\n";
            let truncate_len = truncate_msg.len();
            
            let mut char_count = truncate_len;
            let mut kept_lines = Vec::new();
            
            for line in lines.iter().rev() {
                let line_with_newline = format!("{}\n", line);
                if char_count + line_with_newline.len() > MAX_CHARS {
                    break;
                }
                char_count += line_with_newline.len();
                kept_lines.push(*line);
            }
            
            kept_lines.reverse();
            
            if kept_lines.len() < lines.len() {
                result.push_str(truncate_msg);
            }
            result.push_str(&kept_lines.join("\n"));
            result
        };
        
        let update_message = |log: &str| {
            let keyboard = InlineKeyboardMarkup::new(vec![vec![
                InlineKeyboardButton::callback("🛑 Stop", format!("stop_{}", binary_name_str))
            ]]);
            
            let truncated_log = truncate_log(log);
            let formatted_log = format_output_with_emoji(&truncated_log, Some(&binary_name_str));
            
            let msg_text = format!("🔄 *{}* is running\\.\\.\\.\n\n```\n{}\n```", escape_markdown(&binary_name_str), escape_markdown(&formatted_log));
            
            let final_msg = if msg_text.len() > 4096 {
                let emergency_truncate = "\n... (message too long, showing only most recent) ...\n";
                let available = 4096 - emergency_truncate.len() - 100;
                let truncated = if formatted_log.len() > available {
                    &formatted_log[formatted_log.len().saturating_sub(available)..]
                } else {
                    &formatted_log
                };
                format!("🔄 *{}* is running\\.\\.\\.\n\n```\n{}{}\n```", escape_markdown(&binary_name_str), escape_markdown(emergency_truncate), escape_markdown(truncated))
            } else {
                msg_text
            };
            
            bot_clone.edit_message_text(
                chat_id_clone,
                msg_id,
                final_msg,
            )
            .reply_markup(keyboard)
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        };
        
        let mut last_update = std::time::Instant::now();
        let update_interval = std::time::Duration::from_millis(200);
        let mut pending_update = false;
        
        loop {
            let still_running = {
                let procs = processes_clone.lock().await;
                procs.contains_key(&binary_name_str)
            };
            
            if !still_running {
                if pending_update && !full_log.is_empty() {
                    let _ = update_message(&full_log).await;
                }
                break;
            }
            
            match tokio::time::timeout(std::time::Duration::from_millis(50), rx.recv()).await {
                Ok(Some(line)) => {
                    full_log.push_str(&line);
                    full_log.push('\n');
                    pending_update = true;
                    
                    const MAX_MEMORY_CHARS: usize = 50000;
                    if full_log.len() > MAX_MEMORY_CHARS {
                        let keep_from = full_log.len() - MAX_MEMORY_CHARS;
                        if let Some(newline_pos) = full_log[keep_from..].find('\n') {
                            full_log.drain(..keep_from + newline_pos + 1);
                        } else {
                            full_log.drain(..keep_from);
                        }
                    }
                    
                    if last_update.elapsed() >= update_interval {
                        let _ = update_message(&full_log).await;
                        last_update = std::time::Instant::now();
                        pending_update = false;
                    }
                }
                Ok(None) => {
                    let was_stopped = {
                        let procs = processes_clone.lock().await;
                        !procs.contains_key(&binary_name_str)
                    };
                    
                    if !was_stopped {
                        if !full_log.is_empty() {
                            let _ = update_message(&full_log).await;
                        }
                    }
                    break;
                }
                Err(_) => {
                    if pending_update && last_update.elapsed() >= update_interval && !full_log.is_empty() {
                        let _ = update_message(&full_log).await;
                        last_update = std::time::Instant::now();
                        pending_update = false;
                    }
                    continue;
                }
            }
        }
        
        let was_stopped = {
            let procs = processes_clone.lock().await;
            !procs.contains_key(&binary_name_str)
        };
        
        if !was_stopped {
            let mut procs = processes_clone.lock().await;
            if let Some((mut child, _)) = procs.remove(&binary_name_str) {
                drop(procs);
                let _ = child.wait().await;
            }
            
            let keyboard = InlineKeyboardMarkup::new(vec![vec![
                InlineKeyboardButton::callback("◀️ Back to Menu", "back_to_menu")
            ]]);
            
            let final_message = if full_log.trim().is_empty() {
                format!("✅ *{}* has finished\\.", escape_markdown(&binary_name_str))
            } else {
                let formatted_log = format_output_with_emoji(&full_log, Some(&binary_name_str));
                format!("🔄 *{}* has finished\\.\\.\\.\n\n```\n{}\n```", escape_markdown(&binary_name_str), escape_markdown(&formatted_log))
            };
            
            let _ = bot_clone.edit_message_text(
                chat_id_clone,
                msg_id,
                final_message,
            )
            .reply_markup(keyboard)
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await;
        }
    });
    
    Ok(())
}

async fn handle_stop_process(
    bot: &Bot,
    chat_id: ChatId,
    binary_name: &str,
    processes: ProcessMap,
) -> ResponseResult<()> {
    let process_tuple = {
        let mut procs: tokio::sync::MutexGuard<'_, HashMap<String, (TokioChild, MessageId)>> = processes.lock().await;
        procs.remove(binary_name)
    };
    
    if let Some((mut child, msg_id)) = process_tuple {
        if let Err(e) = child.start_kill() {
            bot.send_message(chat_id, format!("⚠️ Failed to stop *{}*: {}", escape_markdown(binary_name), e))
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
            return Ok(());
        }
        
        let _ = tokio::time::timeout(std::time::Duration::from_secs(5), child.wait()).await;
        
        let keyboard = InlineKeyboardMarkup::new(vec![vec![
            InlineKeyboardButton::callback("◀️ Back to Menu", "back_to_menu")
        ]]);
        
        let _ = bot.edit_message_text(chat_id, msg_id, format!("🛑 *{}* stopped by user\\.", escape_markdown(binary_name)))
            .reply_markup(keyboard)
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await;
    }
    
    Ok(())
}

fn escape_markdown(text: &str) -> String {
    text.replace('*', "\\*")
        .replace('_', "\\_")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('(', "\\(")
        .replace(')', "\\)")
        .replace('~', "\\~")
        .replace('`', "\\`")
        .replace('>', "\\>")
        .replace('#', "\\#")
        .replace('+', "\\+")
        .replace('-', "\\-")
        .replace('=', "\\=")
        .replace('|', "\\|")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('.', "\\.")
        .replace('!', "\\!")
}

fn format_output_with_emoji(text: &str, binary_name: Option<&str>) -> String {
    let mut cleaned = text.to_string();
    loop {
        let start = cleaned.find("\x1b[");
        if let Some(start_pos) = start {
            if let Some(end_pos) = cleaned[start_pos..].find('m') {
                cleaned.replace_range(start_pos..start_pos + end_pos + 1, "");
            } else {
                break;
            }
        } else {
            break;
        }
    }
    
    let lines: Vec<&str> = cleaned.lines().collect();
    let mut processed_lines: Vec<String> = Vec::new();
    let mut skip_next_responding = false;
    let is_health_check = binary_name == Some("health_check");
    
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        if is_health_check {
            if trimmed.contains("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━") ||
               trimmed.contains("POLYMARKET BOT") ||
               trimmed == "CHECK" ||
               trimmed == "SYSTEM" ||
               trimmed.contains("SYSTEM CHECK") ||
               trimmed.is_empty() {
                continue;
            }
        }
        
        if trimmed == "─" || 
           trimmed.starts_with("─") && trimmed.len() > 10 ||
           trimmed.starts_with("╭") && trimmed.contains("─") ||
           trimmed.starts_with("╰") && trimmed.contains("─") {
            continue;
        }
        
        if !is_health_check && trimmed.is_empty() {
            continue;
        }
        
        if is_health_check && trimmed == "responding" {
            if skip_next_responding {
                skip_next_responding = false;
                continue;
            }
            if let Some(last_line) = processed_lines.last_mut() {
                if last_line.contains("RPC:") && !last_line.contains("responding") {
                    if let Some(colon_pos) = last_line.find(':') {
                        *last_line = format!("{}: responding", &last_line[..colon_pos]);
                    }
                    continue;
                } else if (last_line.contains("Polymarket") || last_line.contains("Polymarket API")) && !last_line.contains("responding") {
                    if let Some(colon_pos) = last_line.find(':') {
                        *last_line = format!("{}: responding", &last_line[..colon_pos]);
                    }
                    continue;
                }
            }
            continue;
        }
        skip_next_responding = false;
        
        if trimmed.starts_with("✓") || trimmed.starts_with("✗") || trimmed.starts_with("⚠") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 3 {
                let icon = if parts[0] == "✓" { "✅" } else if parts[0] == "✗" { "🔴" } else { "🟡" };
                let label = parts[1];
                let message = parts[2..].join(" ");
                
                if is_health_check {
                    let next_is_responding = i + 1 < lines.len() && lines[i + 1].trim() == "responding";
                    
                    if label == "RPC" {
                        if next_is_responding {
                            skip_next_responding = true;
                            processed_lines.push(format!("✅   {}: responding", label));
                            continue;
                        } else if message.contains("endpoint") {
                            processed_lines.push(format!("{}   {}: {}", icon, label, message));
                            continue;
                        }
                    }
                    
                    if label == "Polymarket" {
                        if next_is_responding {
                            skip_next_responding = true;
                            processed_lines.push(format!("✅   {} API: responding", label));
                            continue;
                        }
                        let cleaned_msg = message.replace("API API", "").replace("API", "").trim().to_string();
                        if cleaned_msg.is_empty() {
                            processed_lines.push(format!("✅   {} API: {}", label, message.trim()));
                        } else {
                            processed_lines.push(format!("{}   {} API: {}", icon, label, cleaned_msg));
                        }
                        continue;
                    }
                    
                    if label == "Balance" && message.starts_with("Balance:") {
                        let cleaned_msg = message.replace("Balance:", "").trim().to_string();
                        processed_lines.push(format!("{}   {}: {}", icon, label, cleaned_msg));
                        continue;
                    }
                    
                    processed_lines.push(format!("{}   {}: {}", icon, label, message));
                    continue;
                } else {
                    processed_lines.push(format!("{} {}", icon, trimmed.replace("✓", "").replace("✗", "").replace("⚠", "").trim_start()));
                    continue;
                }
            }
        }
        
        if is_health_check && trimmed.contains("Ready to run") {
            let cleaned_ready = trimmed.replace("Ready to run: make run", "Ready to run")
                                       .replace("Ready to run:make run", "Ready to run");
            processed_lines.push(String::new());
            processed_lines.push(String::new());
            processed_lines.push(format!("✅   {}", cleaned_ready));
            continue;
        }
        
        let mut processed_line = line.to_string();
        
        processed_line = processed_line.replace("✓", "✅");
        processed_line = processed_line.replace("✗", "🔴");
        processed_line = processed_line.replace("⚠", "🟡");
        processed_line = processed_line.replace("ℹ", "🔵");
        
        if !processed_line.contains("✅") && !processed_line.contains("🔴") && !processed_line.contains("🟡") {
            let lower = processed_line.to_lowercase();
            if lower.contains("success") || lower.contains("completed") || lower.contains("sufficient") || 
               lower.contains("no action needed") || lower.contains("ready") {
                processed_line = format!("✅ {}", processed_line.trim_start());
            } else if lower.contains("error") || lower.contains("failed") || lower.contains("insufficient") {
                processed_line = format!("🔴 {}", processed_line.trim_start());
            } else if lower.contains("warning") {
                processed_line = format!("🟡 {}", processed_line.trim_start());
            }
        }
        
        if !processed_line.trim().is_empty() {
            processed_lines.push(processed_line);
        }
    }
    
    processed_lines.join("\n")
}

