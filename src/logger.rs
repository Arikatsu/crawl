use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

pub static QUIET_MODE: AtomicBool = AtomicBool::new(false);
static LOGGER: Mutex<()> = Mutex::new(());

pub(crate) fn write_line(msg: &str) {
    let _lock = LOGGER.lock().unwrap();
    eprintln!("{msg}");
}

pub(crate) fn mute() {
    QUIET_MODE.store(true, Ordering::Relaxed);
}

macro_rules! log_info {
    ($($arg:tt)*) => ({
        // probably better to only do this for info only, since warnings and errors should always be visible
        if !$crate::logger::QUIET_MODE.load(std::sync::atomic::Ordering::Relaxed) {
            $crate::logger::write_line(&format!("\x1b[34m    INFO\x1b[0m   {}", format!($($arg)*)));
        }
    })
}

macro_rules! log_warn {
    ($($arg:tt)*) => ({
        $crate::logger::write_line(&format!("\x1b[33m    WARN\x1b[0m   {}", format!($($arg)*)));
    })
}

macro_rules! log_error {
    ($($arg:tt)*) => ({
        $crate::logger::write_line(&format!("\x1b[31m   ERROR\x1b[0m   {}", format!($($arg)*)));
    })
}

pub(crate) use {log_error as error, log_info as info, log_warn as warn};