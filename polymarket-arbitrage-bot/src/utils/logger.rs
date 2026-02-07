use chrono::{DateTime, Utc};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

// Lazy static file paths and handles (FYI: created once, reused)
lazy_static::lazy_static! {
    static ref MONITOR_LOG_PATH: PathBuf = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("monitor.log");
    static ref ERROR_LOG_PATH: PathBuf = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("error.log");
    static ref MONITOR_FILE: Mutex<Option<File>> = Mutex::new(None); // Cached file handle
    static ref ERROR_FILE: Mutex<Option<File>> = Mutex::new(None); // Cached file handle
}

// Ensure log files exist (AFAIK: creates empty files if missing)
fn ensure_log_files() {
    let _ = std::fs::File::create(&*MONITOR_LOG_PATH);
    let _ = std::fs::File::create(&*ERROR_LOG_PATH);
}

// Format timestamp in EST (BTW: UTC-5, no DST handling)
fn format_log_timestamp() -> String {
    let now = Utc::now();
    // Convert to EST (UTC-5)
    let est_offset = chrono::Duration::hours(-5);
    let est_time = now + est_offset;
    est_time.format("%Y-%m-%d %H:%M:%S EST").to_string()
}

// Log price data to monitor.log (FYI: CSV format for easy parsing)
pub fn log_monitor_data(data: MonitorData) {
    ensure_log_files();
    
    let mut file_guard = MONITOR_FILE.lock().unwrap();
    if file_guard.is_none() {
        if let Ok(file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&*MONITOR_LOG_PATH)
        {
            *file_guard = Some(file);
        }
    }
    
    if let Some(ref mut file) = *file_guard {
        let log_line = format!(
            "{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4}\n",
            data.time, data.bid_up, data.bid_down, data.bid_sum,
            data.ask_up, data.ask_down, data.ask_sum
        );
        let _ = file.write_all(log_line.as_bytes());
        let _ = file.flush();
    }
}

pub fn log_error(error: &str, context: Option<&str>) {
    ensure_log_files();
    
    let mut file_guard = ERROR_FILE.lock().unwrap();
    if file_guard.is_none() {
        if let Ok(file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&*ERROR_LOG_PATH)
        {
            *file_guard = Some(file);
        }
    }
    
    if let Some(ref mut file) = *file_guard {
        let timestamp = format_log_timestamp();
        let context_str = context.map(|c| format!(" [{}]", c)).unwrap_or_default();
        let log_line = format!("[{}]{} {}\n", timestamp, context_str, error);
        let _ = file.write_all(log_line.as_bytes());
        let _ = file.flush();
    }
}

pub fn clear_log_files() {
    let header = "Time,Bid UP,Bid DOWN,Bid Sum,Ask UP,Ask DOWN,Ask Sum\n";
    let _ = std::fs::write(&*MONITOR_LOG_PATH, header);
    let _ = std::fs::write(&*ERROR_LOG_PATH, "");
    
    *MONITOR_FILE.lock().unwrap() = None;
    *ERROR_FILE.lock().unwrap() = None;
}

pub fn init_monitor_log() {
    ensure_log_files();
    
    let header = "Time,Bid UP,Bid DOWN,Bid Sum,Ask UP,Ask DOWN,Ask Sum\n";
    if let Ok(content) = std::fs::read_to_string(&*MONITOR_LOG_PATH) {
        if content.trim().is_empty() {
            let _ = std::fs::write(&*MONITOR_LOG_PATH, header);
        }
    }
}

#[derive(Debug, Clone)]
pub struct MonitorData {
    pub time: String,
    pub bid_up: f64,
    pub bid_down: f64,
    pub bid_sum: f64,
    pub ask_up: f64,
    pub ask_down: f64,
    pub ask_sum: f64,
}

