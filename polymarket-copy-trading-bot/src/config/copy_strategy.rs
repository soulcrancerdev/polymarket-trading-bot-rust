use anyhow::{Context, Result};
use std::cmp::Ordering;

// Copy strategy types: percentage, fixed USD, or adaptive
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyStrategy {
    Percentage,
    Fixed,
    Adaptive,
}

// Multiplier tier (for tiered multipliers based on trade size)
#[derive(Debug, Clone)]
pub struct MultiplierTier {
    pub min: f64,
    pub max: Option<f64>,
    pub multiplier: f64,
}

// Config for copy trading strategy (size limits, multipliers, etc)
#[derive(Debug, Clone)]
pub struct CopyStrategyConfig {
    pub strategy: CopyStrategy,
    pub copy_size: f64,
    pub max_order_size_usd: f64,
    pub min_order_size_usd: f64,
    pub max_position_size_usd: Option<f64>,
    pub max_daily_volume_usd: Option<f64>,
    pub adaptive_min_percent: Option<f64>,
    pub adaptive_max_percent: Option<f64>,
    pub adaptive_threshold: Option<f64>,
    pub tiered_multipliers: Option<Vec<MultiplierTier>>,
    pub trade_multiplier: Option<f64>,
}

// Result of order size calculation (with reasoning)
#[derive(Debug, Clone)]
pub struct OrderSizeCalculation {
    pub trader_order_size: f64,
    pub base_amount: f64,
    pub final_amount: f64,
    pub strategy: CopyStrategy,
    pub capped_by_max: bool,
    pub reduced_by_balance: bool,
    pub below_minimum: bool,
    pub reasoning: String,
}

// Linear interpolation helper
fn lerp(a: f64, b: f64, t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    a + (b - a) * t
}

// Calc adaptive percentage (adjusts based on trader's order size)
fn calculate_adaptive_percent(config: &CopyStrategyConfig, trader_order_size: f64) -> f64 {
    let min_pct = config.adaptive_min_percent.unwrap_or(config.copy_size);
    let max_pct = config.adaptive_max_percent.unwrap_or(config.copy_size);
    let threshold = config.adaptive_threshold.unwrap_or(500.0);

    // Larger trades = lower %, smaller trades = higher %
    if trader_order_size >= threshold {
        let factor = (trader_order_size / threshold - 1.0).min(1.0);
        lerp(config.copy_size, min_pct, factor)
    } else {
        let factor = trader_order_size / threshold;
        lerp(max_pct, config.copy_size, factor)
    }
}

// Get multiplier for trade (tiered or flat)
pub fn get_trade_multiplier(config: &CopyStrategyConfig, trader_order_size: f64) -> f64 {
    // Check tiered multipliers first
    if let Some(ref tiers) = config.tiered_multipliers {
        if !tiers.is_empty() {
            for tier in tiers {
                if trader_order_size >= tier.min {
                    match tier.max {
                        None => return tier.multiplier,
                        Some(max) if trader_order_size < max => return tier.multiplier,
                        _ => continue,
                    }
                }
            }
            return tiers.last().map(|t| t.multiplier).unwrap_or(1.0);
        }
    }
    // Fallback to flat multiplier
    config.trade_multiplier.unwrap_or(1.0)
}

// Calculate order size based on strategy, limits, & balance
pub fn calculate_order_size(
    config: &CopyStrategyConfig,
    trader_order_size: f64,
    available_balance: f64,
    current_position_size: f64,
) -> OrderSizeCalculation {
    // Calc base amount based on strategy
    let (base_amount, strategy, mut reasoning) = match config.strategy {
        CopyStrategy::Percentage => {
            let base = trader_order_size * (config.copy_size / 100.0);
            let r = format!(
                "{}% of trader's ${:.2} = ${:.2}",
                config.copy_size, trader_order_size, base
            );
            (base, CopyStrategy::Percentage, r)
        }
        CopyStrategy::Fixed => {
            let r = format!("Fixed amount: ${:.2}", config.copy_size);
            (config.copy_size, CopyStrategy::Fixed, r)
        }
        CopyStrategy::Adaptive => {
            let pct = calculate_adaptive_percent(config, trader_order_size);
            let base = trader_order_size * (pct / 100.0);
            let r = format!(
                "Adaptive {:.1}% of trader's ${:.2} = ${:.2}",
                pct, trader_order_size, base
            );
            (base, CopyStrategy::Adaptive, r)
        }
    };

    // Apply multiplier (tiered or flat)
    let multiplier = get_trade_multiplier(config, trader_order_size);
    let mut final_amount = base_amount * multiplier;
    if (multiplier - 1.0).abs() > 1e-9 {
        reasoning.push_str(&format!(
            " → {}x multiplier: ${:.2} → ${:.2}",
            multiplier, base_amount, final_amount
        ));
    }

    // Track if we hit limits (for logging)
    let mut capped_by_max = false;
    let mut reduced_by_balance = false;
    let mut below_minimum = false;

    if final_amount > config.max_order_size_usd {
        final_amount = config.max_order_size_usd;
        capped_by_max = true;
        reasoning.push_str(&format!(" → Capped at max ${}", config.max_order_size_usd));
    }

    if let Some(max_pos) = config.max_position_size_usd {
        let new_total = current_position_size + final_amount;
        if new_total > max_pos {
            let allowed = (max_pos - current_position_size).max(0.0);
            if allowed < config.min_order_size_usd {
                final_amount = 0.0;
                reasoning.push_str(" → Position limit reached");
            } else {
                final_amount = allowed;
                reasoning.push_str(" → Reduced to fit position limit");
            }
        }
    }

    let max_affordable = available_balance * 0.99;
    if final_amount > max_affordable {
        final_amount = max_affordable;
        reduced_by_balance = true;
        reasoning.push_str(&format!(
            " → Reduced to fit balance (${:.2})",
            max_affordable
        ));
    }

    if final_amount < config.min_order_size_usd {
        below_minimum = true;
        reasoning.push_str(&format!(" → Below minimum ${}", config.min_order_size_usd));
        final_amount = config.min_order_size_usd;
    }

    OrderSizeCalculation {
        trader_order_size,
        base_amount,
        final_amount,
        strategy,
        capped_by_max,
        reduced_by_balance,
        below_minimum,
        reasoning,
    }
}

pub fn parse_tiered_multipliers(tiers_str: &str) -> Result<Vec<MultiplierTier>> {
    let trimmed = tiers_str.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let mut tiers = Vec::new();
    for part in trimmed.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let mut split = part.split(':');
        let range = split.next().context("Invalid tier: missing range")?.trim();
        let mult_str = split
            .next()
            .context("Invalid tier: missing multiplier")?
            .trim();
        let multiplier: f64 = mult_str.parse().context("Invalid multiplier")?;
        if multiplier < 0.0 {
            anyhow::bail!("Invalid multiplier in tier: {}", part);
        }
        if range.ends_with('+') {
            let min: f64 = range[..range.len() - 1]
                .trim()
                .parse()
                .context("Invalid min in tier")?;
            if min < 0.0 {
                anyhow::bail!("Invalid minimum in tier: {}", part);
            }
            tiers.push(MultiplierTier {
                min,
                max: None,
                multiplier,
            });
        } else if let Some((min_s, max_s)) = range.split_once('-') {
            let min: f64 = min_s.trim().parse().context("Invalid min")?;
            let max: f64 = max_s.trim().parse().context("Invalid max")?;
            if min < 0.0 {
                anyhow::bail!("Invalid minimum in tier: {}", part);
            }
            if max <= min {
                anyhow::bail!("max must be > min in tier: {}", part);
            }
            tiers.push(MultiplierTier {
                min,
                max: Some(max),
                multiplier,
            });
        } else {
            anyhow::bail!("Invalid range format in tier: {}", part);
        }
    }
    tiers.sort_by(|a, b| a.min.partial_cmp(&b.min).unwrap_or(Ordering::Equal));
    for i in 0..tiers.len().saturating_sub(1) {
        let cur = &tiers[i];
        let next = &tiers[i + 1];
        if cur.max.is_none() {
            anyhow::bail!("Tier with infinite upper bound must be last");
        }
        if let Some(cur_max) = cur.max {
            if cur_max > next.min {
                anyhow::bail!("Overlapping tiers");
            }
        }
    }
    Ok(tiers)
}
