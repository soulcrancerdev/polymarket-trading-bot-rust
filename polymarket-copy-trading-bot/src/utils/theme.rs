pub mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";

    pub const ACCENT: &str = "\x1b[38;5;51m";
    pub const ACCENT_BOLD: &str = "\x1b[1;38;5;51m";
    pub const MINT: &str = "\x1b[38;5;85m";
    pub const SUCCESS: &str = "\x1b[38;5;46m";
    pub const WARN: &str = "\x1b[38;5;214m";
    pub const ERROR: &str = "\x1b[38;5;196m";
    pub const MUTED: &str = "\x1b[38;5;245m";
    pub const HIGHLIGHT: &str = "\x1b[38;5;213m";
    pub const GOLD: &str = "\x1b[38;5;220m";
    pub const BOX: &str = "\x1b[38;5;33m";
}

pub mod icons {
    pub const INFO: &str = "●";
    pub const OK: &str = "✓";
    pub const WARN: &str = "⚠";
    pub const ERR: &str = "✗";
    pub const ARROW: &str = "▸";
    pub const DOT: &str = "·";
    pub const TRADE: &str = "◆";
    pub const VAULT: &str = "◇";
}

pub fn panel_top(width: usize) -> String {
    format!(
        "{}╭{}╮{}",
        colors::BOX,
        "─".repeat(width.saturating_sub(2)),
        colors::RESET
    )
}

pub fn panel_bottom(width: usize) -> String {
    format!(
        "{}╰{}╯{}",
        colors::BOX,
        "─".repeat(width.saturating_sub(2)),
        colors::RESET
    )
}

#[rustfmt::skip]
pub const BANNER: &[&str] = &[
    "  ██████╗  ██████╗ ██╗  ██╗   ██╗",
    "  ██╔══██╗██╔═══██╗██║  ╚██╗ ██╔╝",
    "  ██████╔╝██║   ██║██║   ╚████╔╝ ",
    "  ██╔═══╝ ██║   ██║██║    ╚██╔╝  ",
    "  ██║     ╚██████╔╝███████╗██║   ",
    "  ╚═╝      ╚═════╝ ╚══════╝╚═╝   ",
];
