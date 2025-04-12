use std::fs::{File, OpenOptions, create_dir_all};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::timestamp::{now};
use std::io::Result as IoResult;


pub static LOG_FILE: Lazy<Mutex<Option<File>>> = Lazy::new(|| Mutex::new(None));

pub fn prepare_log_file() -> IoResult<()> {
    let mut log_path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    log_path.push("WindowsWatcher");

    create_dir_all(&log_path)?;
    log_path.push("windows_watcher.log");

    init_log_file(log_path)?;
    Ok(())
}


fn init_log_file(path: PathBuf) -> std::io::Result<()> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;

    *LOG_FILE.lock().map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Log mutex poisoned"))? = Some(file);
    Ok(())
}

pub fn log_message(message: &str) {
    let timestamp = now();
    let formatted_message = format!("[{}] {}", timestamp, message);

    println!("{}", formatted_message);

    if let Ok(mut guard) = LOG_FILE.lock() {
        if let Some(file) = guard.as_mut() {
            let _ = writeln!(file, "{}", formatted_message);
        }
    }
}
