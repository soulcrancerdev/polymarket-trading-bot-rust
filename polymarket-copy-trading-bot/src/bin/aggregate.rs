use anyhow::Result;
use polymarket_copy_rust::utils::theme::colors;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TraderResult {
    address: Option<String>,
    roi: Option<f64>,
    total_pnl: Option<f64>,
    #[serde(rename = "totalPnl")]
    total_pnl_camel: Option<f64>,
    win_rate: Option<f64>,
    #[serde(rename = "winRate")]
    win_rate_camel: Option<f64>,
    copied_trades: Option<u32>,
    #[serde(rename = "copiedTrades")]
    copied_trades_camel: Option<u32>,
    status: Option<String>,
}

impl TraderResult {
    fn roi(&self) -> Option<f64> {
        self.roi
    }

    fn total_pnl(&self) -> f64 {
        self.total_pnl.or(self.total_pnl_camel).unwrap_or(0.0)
    }

    fn win_rate(&self) -> f64 {
        self.win_rate.or(self.win_rate_camel).unwrap_or(0.0)
    }

    fn address(&self) -> Option<&str> {
        self.address.as_deref()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScanResult {
    scan_date: Option<String>,
    #[serde(rename = "scanDate")]
    scan_date_camel: Option<String>,
    config: Config,
    summary: Option<Summary>,
    traders: Option<Vec<TraderResult>>,
    results: Option<Vec<TraderResult>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    history_days: Option<u32>,
    #[serde(rename = "historyDays")]
    history_days_camel: Option<u32>,
    multiplier: Option<f64>,
    min_order_size: Option<f64>,
    #[serde(rename = "minOrderSize")]
    min_order_size_camel: Option<f64>,
    starting_capital: Option<f64>,
    #[serde(rename = "startingCapital")]
    starting_capital_camel: Option<f64>,
}

impl Config {
    fn history_days(&self) -> Option<u32> {
        self.history_days.or(self.history_days_camel)
    }

    fn multiplier(&self) -> f64 {
        self.multiplier.unwrap_or(1.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Summary {
    total_analyzed: Option<u32>,
    profitable: Option<u32>,
    avg_roi: Option<f64>,
    avg_win_rate: Option<f64>,
}

#[derive(Debug, Clone)]
struct StrategyPerformance {
    strategy_id: String,
    history_days: u32,
    multiplier: f64,
    best_roi: f64,
    best_win_rate: f64,
    best_pnl: f64,
    avg_roi: f64,
    avg_win_rate: f64,
    traders_analyzed: u32,
    profitable_traders: u32,
    files_count: u32,
}

#[derive(Debug, Clone)]
struct TraderData {
    best_roi: f64,
    best_strategy: String,
    times_found: u32,
}

fn main() -> Result<()> {
    println!();
    println!(
        "{}{}{}",
        colors::ACCENT,
        "╔════════════════════════════════════════════════════════════════╗",
        colors::RESET
    );
    println!(
        "{}║          📊 АГРЕГАТОР РЕЗУЛЬТАТОВ ВСЕХ СТРАТЕГИЙ              ║{}",
        colors::ACCENT,
        colors::RESET
    );
    println!(
        "{}{}{}",
        colors::ACCENT,
        "╚════════════════════════════════════════════════════════════════╝",
        colors::RESET
    );
    println!();

    let dirs = vec![
        "trader_scan_results",
        "trader_analysis_results",
        "top_traders_results",
        "strategy_factory_results",
    ];

    let mut all_strategies: HashMap<String, StrategyPerformance> = HashMap::new();
    let mut all_traders: HashMap<String, TraderData> = HashMap::new();
    let mut total_files = 0;

    for dir in &dirs {
        let dir_path = PathBuf::from(dir);
        if !dir_path.exists() {
            continue;
        }

        let files: Vec<_> = fs::read_dir(&dir_path)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s == "json")
                    .unwrap_or(false)
            })
            .collect();

        println!(
            "{}📁 Сканирование {}/: найдено {} файлов{}",
            colors::MUTED,
            dir,
            files.len(),
            colors::RESET
        );

        for entry in files {
            total_files += 1;
            let file_path = entry.path();

            if let Ok(content) = fs::read_to_string(&file_path) {
                if let Ok(data) = serde_json::from_str::<ScanResult>(&content) {
                    let config = data.config;
                    let history_days = match config.history_days() {
                        Some(d) => d,
                        None => continue,
                    };

                    let strategy_id = format!("{}d_{}x", history_days, config.multiplier());

                    all_strategies
                        .entry(strategy_id.clone())
                        .or_insert_with(|| StrategyPerformance {
                            strategy_id: strategy_id.clone(),
                            history_days,
                            multiplier: config.multiplier(),
                            best_roi: f64::NEG_INFINITY,
                            best_win_rate: 0.0,
                            best_pnl: f64::NEG_INFINITY,
                            avg_roi: 0.0,
                            avg_win_rate: 0.0,
                            traders_analyzed: 0,
                            profitable_traders: 0,
                            files_count: 0,
                        });

                    let strategy = all_strategies.get_mut(&strategy_id).unwrap();
                    strategy.files_count += 1;

                    let traders = data.traders.or(data.results).unwrap_or_default();
                    let mut total_roi = 0.0;
                    let mut total_win_rate = 0.0;
                    let mut traders_count = 0;

                    for trader in traders {
                        let roi = match trader.roi() {
                            Some(r) => r,
                            None => continue,
                        };

                        traders_count += 1;
                        total_roi += roi;
                        total_win_rate += trader.win_rate();

                        if roi > strategy.best_roi {
                            strategy.best_roi = roi;
                        }
                        if trader.win_rate() > strategy.best_win_rate {
                            strategy.best_win_rate = trader.win_rate();
                        }
                        let pnl = trader.total_pnl();
                        if pnl > strategy.best_pnl {
                            strategy.best_pnl = pnl;
                        }
                        if roi > 0.0 {
                            strategy.profitable_traders += 1;
                        }

                        if let Some(addr) = trader.address() {
                            all_traders
                                .entry(addr.to_string())
                                .and_modify(|t| {
                                    t.times_found += 1;
                                    if roi > t.best_roi {
                                        t.best_roi = roi;
                                        t.best_strategy = strategy_id.clone();
                                    }
                                })
                                .or_insert_with(|| TraderData {
                                    best_roi: roi,
                                    best_strategy: strategy_id.clone(),
                                    times_found: 1,
                                });
                        }
                    }

                    strategy.traders_analyzed += traders_count;
                    if traders_count > 0 {
                        strategy.avg_roi = total_roi / traders_count as f64;
                        strategy.avg_win_rate = total_win_rate / traders_count as f64;
                    }
                }
            }
        }
    }

    println!(
        "{}✓ Обработано {} файлов{}\n",
        colors::SUCCESS,
        total_files,
        colors::RESET
    );

    let mut strategies: Vec<_> = all_strategies.values().cloned().collect();
    strategies.sort_by(|a, b| b.best_roi.partial_cmp(&a.best_roi).unwrap());

    println!("{}{}{}", colors::ACCENT, "═".repeat(100), colors::RESET);
    println!("{}  🏆 ТОП СТРАТЕГИЙ ПО ЛУЧШЕМУ ROI{}", colors::ACCENT, colors::RESET);
    println!("{}{}{}\n", colors::ACCENT, "═".repeat(100), colors::RESET);

    println!(
        "{}  #  | Strategy      | Best ROI  | Best Win% | Best P&L   | Avg ROI   | Profitable | Files{}",
        colors::BOLD,
        colors::RESET
    );
    println!("{}{}{}", colors::MUTED, "─".repeat(100), colors::RESET);

    for (i, s) in strategies.iter().take(15).enumerate() {
        let roi_color = if s.best_roi >= 0.0 {
            colors::SUCCESS
        } else {
            colors::ERROR
        };
        let roi_sign = if s.best_roi >= 0.0 { "+" } else { "" };
        let pnl_sign = if s.best_pnl >= 0.0 { "+" } else { "" };

        let roi_str = format!("{}{:.1}%", roi_sign, s.best_roi);
        let pnl_str = format!("{}{:.0}", pnl_sign, s.best_pnl);
        let win_rate_str = format!("{:.1}%", s.best_win_rate);
        let avg_roi_str = format!("{:.1}%", s.avg_roi);
        let profitable_str = format!("{}/{}", s.profitable_traders, s.traders_analyzed);
        println!(
            "  {}{:2}{} | {}{:13}{} | {}{:9}{} | {}{:9}{} | ${:9}{} | {:9} | {:10} | {}",
            colors::WARN,
            i + 1,
            colors::RESET,
            colors::ACCENT,
            s.strategy_id,
            colors::RESET,
            roi_color,
            roi_str,
            colors::RESET,
            colors::WARN,
            win_rate_str,
            colors::RESET,
            pnl_str,
            colors::RESET,
            avg_roi_str,
            profitable_str,
            s.files_count
        );
    }

    println!("\n{}{}{}", colors::ACCENT, "═".repeat(100), colors::RESET);
    println!(
        "{}  🎯 ТОП ТРЕЙДЕРОВ (найдены в нескольких сканах){}",
        colors::ACCENT,
        colors::RESET
    );
    println!("{}{}{}\n", colors::ACCENT, "═".repeat(100), colors::RESET);

    let mut top_traders: Vec<_> = all_traders.iter().collect();
    top_traders.sort_by(|(_, a), (_, b)| b.best_roi.partial_cmp(&a.best_roi).unwrap());

    println!(
        "{}  #  | Address                                    | Best ROI  | Best Strategy | Найден раз{}",
        colors::BOLD,
        colors::RESET
    );
    println!("{}{}{}", colors::MUTED, "─".repeat(100), colors::RESET);

    for (i, (address, data)) in top_traders.iter().take(10).enumerate() {
        let roi_color = if data.best_roi >= 0.0 {
            colors::SUCCESS
        } else {
            colors::ERROR
        };
        let roi_sign = if data.best_roi >= 0.0 { "+" } else { "" };

        let roi_str = format!("{}{:.1}%", roi_sign, data.best_roi);
        println!(
            "  {}{:2}{} | {}{:42}{} | {}{:9}{} | {}{:13}{} | {}",
            colors::WARN,
            i + 1,
            colors::RESET,
            colors::ACCENT,
            address,
            colors::RESET,
            roi_color,
            roi_str,
            colors::RESET,
            colors::ACCENT,
            data.best_strategy,
            colors::RESET,
            data.times_found
        );
    }

    println!("\n{}{}{}", colors::ACCENT, "═".repeat(100), colors::RESET);
    println!("{}  📈 ОБЩАЯ СТАТИСТИКА{}", colors::ACCENT, colors::RESET);
    println!("{}{}{}\n", colors::ACCENT, "═".repeat(100), colors::RESET);

    let total_traders: u32 = strategies.iter().map(|s| s.traders_analyzed).sum();
    let total_profitable: u32 = strategies.iter().map(|s| s.profitable_traders).sum();
    let unique_traders = all_traders.len();
    let profitable_rate = if total_traders > 0 {
        (total_profitable as f64 / total_traders as f64) * 100.0
    } else {
        0.0
    };

    println!("  Всего файлов:           {}{}{}", colors::ACCENT, total_files, colors::RESET);
    println!(
        "  Всего стратегий:        {}{}{}",
        colors::ACCENT,
        strategies.len(),
        colors::RESET
    );
    println!(
        "  Всего трейдеров:        {}{}{}",
        colors::ACCENT,
        total_traders,
        colors::RESET
    );
    println!(
        "  Уникальных трейдеров:   {}{}{}",
        colors::ACCENT,
        unique_traders,
        colors::RESET
    );
    println!(
        "  Прибыльных трейдеров:   {}{} ({:.1}%){}",
        colors::SUCCESS,
        total_profitable,
        profitable_rate,
        colors::RESET
    );

    if let Some(best) = strategies.first() {
        println!("\n{}🌟 ЛУЧШАЯ СТРАТЕГИЯ:{}", colors::SUCCESS, colors::RESET);
        println!("  ID: {}{}{}", colors::WARN, best.strategy_id, colors::RESET);
        println!(
            "  ROI: {}+{:.2}%{}",
            colors::SUCCESS,
            best.best_roi,
            colors::RESET
        );
        println!(
            "  Win Rate: {}{:.1}%{}",
            colors::WARN,
            best.best_win_rate,
            colors::RESET
        );
        println!(
            "  P&L: {}+${:.2}{}",
            colors::SUCCESS,
            best.best_pnl,
            colors::RESET
        );
    }

    let output_dir = PathBuf::from("strategy_factory_results");
    fs::create_dir_all(&output_dir)?;
    let output_path = output_dir.join("aggregated_results.json");

    #[derive(Serialize)]
    struct Output {
        timestamp: String,
        summary: OutputSummary,
        strategies: Vec<StrategyOutput>,
        top_traders: Vec<TraderOutput>,
    }

    #[derive(Serialize)]
    struct OutputSummary {
        total_files: u32,
        total_strategies: usize,
        total_traders: u32,
        unique_traders: usize,
        profitable_traders: u32,
        profitable_rate: f64,
    }

    #[derive(Serialize)]
    struct StrategyOutput {
        strategy_id: String,
        history_days: u32,
        multiplier: f64,
        best_roi: f64,
        best_win_rate: f64,
        best_pnl: f64,
        avg_roi: f64,
        avg_win_rate: f64,
        traders_analyzed: u32,
        profitable_traders: u32,
        files_count: u32,
    }

    #[derive(Serialize)]
    struct TraderOutput {
        address: String,
        best_roi: f64,
        best_strategy: String,
        times_found: u32,
    }

    let output = Output {
        timestamp: chrono::Utc::now().to_rfc3339(),
        summary: OutputSummary {
            total_files,
            total_strategies: strategies.len(),
            total_traders,
            unique_traders,
            profitable_traders: total_profitable,
            profitable_rate,
        },
        strategies: strategies
            .iter()
            .take(20)
            .map(|s| StrategyOutput {
                strategy_id: s.strategy_id.clone(),
                history_days: s.history_days,
                multiplier: s.multiplier,
                best_roi: s.best_roi,
                best_win_rate: s.best_win_rate,
                best_pnl: s.best_pnl,
                avg_roi: s.avg_roi,
                avg_win_rate: s.avg_win_rate,
                traders_analyzed: s.traders_analyzed,
                profitable_traders: s.profitable_traders,
                files_count: s.files_count,
            })
            .collect(),
        top_traders: top_traders
            .iter()
            .take(10)
            .map(|(address, data)| TraderOutput {
                address: (*address).clone(),
                best_roi: data.best_roi,
                best_strategy: data.best_strategy.clone(),
                times_found: data.times_found,
            })
            .collect(),
    };

    fs::write(&output_path, serde_json::to_string_pretty(&output)?)?;
    println!(
        "\n{}✓ Агрегированные результаты сохранены:{} {}{}\n",
        colors::SUCCESS,
        colors::RESET,
        colors::ACCENT,
        output_path.display()
    );

    Ok(())
}
