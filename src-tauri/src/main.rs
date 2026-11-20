// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "linux")]
    configure_gstreamer_plugins();
    r_tube_lib::run()
}

/// WebKitGTK decodes audio and video through GStreamer. The AppImage ships
/// GStreamer libraries but not the plugins, and the library's compiled-in
/// plugin directory is the build machine's, which may not exist here — the
/// WebProcess then finds no decoder and playback stalls on the first frame.
/// Point GStreamer at the host's plugins unless the user configured a path.
#[cfg(target_os = "linux")]
fn configure_gstreamer_plugins() {
    use std::path::Path;

    if std::env::var_os("GST_PLUGIN_SYSTEM_PATH_1_0").is_some()
        || std::env::var_os("GST_PLUGIN_SYSTEM_PATH").is_some()
    {
        return;
    }

    const CANDIDATES: &[&str] = &[
        "/usr/lib/gstreamer-1.0",
        "/usr/lib64/gstreamer-1.0",
        "/usr/lib/x86_64-linux-gnu/gstreamer-1.0",
        "/usr/lib/aarch64-linux-gnu/gstreamer-1.0",
        "/usr/local/lib/gstreamer-1.0",
    ];
    let dirs: Vec<&str> = CANDIDATES
        .iter()
        .copied()
        .filter(|dir| Path::new(dir).is_dir())
        .collect();
    if dirs.is_empty() {
        return;
    }

    // SAFETY: this runs from `main` before Tauri starts its runtime, so no
    // other thread can be reading the environment yet.
    unsafe {
        std::env::set_var("GST_PLUGIN_SYSTEM_PATH_1_0", dirs.join(":"));
    }
}