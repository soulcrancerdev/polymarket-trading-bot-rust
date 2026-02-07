use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

static SPINNER_INDEX: AtomicUsize = AtomicUsize::new(0);

const SPINNER_FRAMES: &[&str] = &[
    "▰▱▱▱▱▱▱",
    "▰▰▱▱▱▱▱",
    "▰▰▰▱▱▱▱",
    "▰▰▰▰▱▱▱",
    "▰▰▰▰▰▱▱",
    "▰▰▰▰▰▰▱",
    "▰▰▰▰▰▰▰",
    "▱▱▱▱▱▱▱",
];

pub struct Spinner;

impl Spinner {
    pub fn frame() -> &'static str {
        let idx = SPINNER_INDEX.fetch_add(1, Ordering::Relaxed);
        SPINNER_FRAMES[idx % SPINNER_FRAMES.len()]
    }
    
    pub fn interval() -> Duration {
        Duration::from_millis(200)
    }
    
    pub fn reset() {
        SPINNER_INDEX.store(0, Ordering::Relaxed);
    }
}

