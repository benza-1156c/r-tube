use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// Returns the base directory for logs.
/// In development, saves to `./logs` relative to the project root.
pub fn get_log_dir() -> PathBuf {
    let current = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let log_dir = if current.join("src-tauri").exists() {
        current.join("logs")
    } else if current.ends_with("src-tauri") {
        current.join("../logs")
    } else {
        current.join("logs")
    };

    let _ = fs::create_dir_all(&log_dir);
    log_dir
}

/// Resolves the absolute path for a log file inside the `logs` folder.
pub fn resolve_log_path(filename: &str, default_ext: &str) -> PathBuf {
    let base_dir = get_log_dir();
    let file_name = if filename.contains('.') {
        filename.to_string()
    } else {
        format!("{filename}.{default_ext}")
    };
    let path = base_dir.join(file_name);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    path
}

/// Dumps any serializable data as a pretty JSON file.
///
/// # Example
/// ```rust
/// dump_json("raw_youtube_feed.json", &raw_json);
/// ```
pub fn dump_json<T: Serialize + ?Sized>(filename: &str, data: &T) -> Result<PathBuf, String> {
    let file_path = resolve_log_path(filename, "json");
    let pretty_str = serde_json::to_string_pretty(data)
        .map_err(|e| format!("Failed to serialize data to JSON: {e}"))?;

    fs::write(&file_path, pretty_str)
        .map_err(|e| format!("Failed to write to file {:?}: {e}", file_path))?;

    println!("[Log] Dumped JSON: {}", file_path.display());
    Ok(file_path)
}

/// Writes any text or string slice to a file (overwrites existing file).
///
/// # Example
/// ```rust
/// write_file("debug.txt", "Some debug message");
/// ```
pub fn write_file(filename: &str, content: impl AsRef<str>) -> Result<PathBuf, String> {
    let file_path = resolve_log_path(filename, "log");
    fs::write(&file_path, content.as_ref())
        .map_err(|e| format!("Failed to write to file {:?}: {e}", file_path))?;

    println!("[Log] Wrote file: {}", file_path.display());
    Ok(file_path)
}

/// Appends a line with a timestamp to a log file.
///
/// # Example
/// ```rust
/// append_file("activity.log", "User logged in");
/// ```
pub fn append_file(filename: &str, content: impl AsRef<str>) -> Result<PathBuf, String> {
    let file_path = resolve_log_path(filename, "log");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path)
        .map_err(|e| format!("Failed to open log file {:?}: {e}", file_path))?;

    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    writeln!(file, "[{timestamp}] {}", content.as_ref())
        .map_err(|e| format!("Failed to append to log file: {e}"))?;

    Ok(file_path)
}

/// Dumps any data that implements `Debug` into a file.
///
/// # Example
/// ```rust
/// dump_debug("state.txt", &my_struct);
/// ```
pub fn dump_debug<T: std::fmt::Debug + ?Sized>(
    filename: &str,
    data: &T,
) -> Result<PathBuf, String> {
    let file_path = resolve_log_path(filename, "txt");
    let content = format!("{data:#?}");
    fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write to file {:?}: {e}", file_path))?;

    println!("[Log] Dumped debug: {}", file_path.display());
    Ok(file_path)
}
